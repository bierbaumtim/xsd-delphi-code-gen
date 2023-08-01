use std::{borrow::Cow, collections::HashMap, fs::File, io::BufReader, path::Path};

use quick_xml::{events::BytesStart, events::Event, Reader};

use crate::parser_types::*;
use crate::type_registry::*;

#[derive(Default)]
pub(crate) struct Parser {
    current_namespace: Option<String>,
    namespace_aliases: HashMap<String, String>,
}

impl Parser {
    pub(crate) fn parse_file<P: AsRef<Path>>(
        &mut self,
        path: P,
        registry: &mut TypeRegistry,
    ) -> Result<Vec<Node>, ParserError> {
        let mut reader = match Reader::from_file(path) {
            Ok(r) => r,
            Err(_) => return Err(ParserError::UnableToReadFile),
        };

        self.parse_nodes(&mut reader, registry)
    }

    pub(crate) fn parse_files<P: AsRef<Path>>(
        &mut self,
        paths: &Vec<P>,
        registry: &mut TypeRegistry,
    ) -> Result<Vec<Node>, ParserError> {
        let mut nodes = Vec::new();

        for path in paths {
            let mut reader = match Reader::from_file(path) {
                Ok(r) => r,
                Err(_) => return Err(ParserError::UnableToReadFile),
            };

            self.current_namespace = None;
            self.namespace_aliases.clear();

            let file_nodes = self.parse_nodes(&mut reader, registry)?;
            nodes.extend(file_nodes);
        }

        Ok(nodes)
    }

    fn parse_nodes(
        &mut self,
        reader: &mut Reader<BufReader<File>>,
        registry: &mut TypeRegistry,
    ) -> Result<Vec<Node>, ParserError> {
        let mut nodes = Vec::new();
        let mut buf = Vec::new();

        let mut current_element_name = None::<String>;

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Eof) => break,
                Ok(Event::Start(s)) => {
                    match s.name().as_ref() {
                        b"xs:schema" => {
                            self.current_namespace =
                                Self::get_attribute_value(&s, "targetNamespace").ok();

                            self.extract_schema_namespace_aliases(&s);
                        }
                        b"xs:element" => {
                            current_element_name = Some(Self::get_attribute_value(&s, "name")?)
                        }
                        b"xs:complexType" => {
                            if let Some(name) = &current_element_name {
                                let c_type =
                                    self.parse_complex_type(reader, registry, name.clone(), true)?;

                                let c_type = CustomTypeDefinition::Complex(c_type);
                                registry.register_type(c_type);

                                let base_attributes = Self::get_base_attributes(&s)?;

                                let node = Node::new(
                                    NodeType::Custom(name.clone()),
                                    name.clone(),
                                    base_attributes,
                                );
                                nodes.push(node);
                            } else {
                                let name = Self::get_attribute_value(&s, "name")
                                    .ok()
                                    .unwrap_or_else(|| registry.generate_type_name());

                                let c_type =
                                    self.parse_complex_type(reader, registry, name, false)?;

                                let c_type = CustomTypeDefinition::Complex(c_type);

                                registry.register_type(c_type);
                            }
                        }
                        b"xs:simpleType" => {
                            if let Some(name) = &current_element_name {
                                let (s_type, type_name) =
                                    self.parse_simple_type(reader, name.clone(), true)?;

                                registry.register_type(s_type);

                                let base_attributes = Self::get_base_attributes(&s)?;

                                let node = Node::new(
                                    NodeType::Custom(type_name),
                                    name.clone(),
                                    base_attributes,
                                );
                                nodes.push(node);
                            } else {
                                let name = Self::get_attribute_value(&s, "name")
                                    .ok()
                                    .unwrap_or_else(|| registry.generate_type_name());

                                let (s_type, _) = self.parse_simple_type(reader, name, false)?;

                                registry.register_type(s_type);
                            }
                        }
                        _ => (),
                    }
                    //
                }
                Ok(Event::End(e)) => {
                    if let b"xs:element" = e.name().as_ref() {
                        current_element_name = None
                    }
                }
                Ok(Event::Empty(e)) => {
                    if let b"xs:element" = e.name().as_ref() {
                        let name = Self::get_attribute_value(&e, "name")?;
                        let b_type = Self::get_attribute_value(&e, "type")?;
                        let b_type = self.resolve_namespace(b_type)?;
                        let node_type = match Self::base_type_str_to_node_type(b_type.as_str()) {
                            Some(t) => t,
                            None => return Err(ParserError::MissingOrNotSupportedBaseType(b_type)),
                        };
                        let base_attributes = Self::get_base_attributes(&e)?;
                        let node = Node::new(node_type, name, base_attributes);
                        nodes.push(node);
                    }
                }
                // There are several other `Event`s we do not consider here
                _ => (),
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }

        Ok(nodes)
    }

    fn parse_complex_type(
        &self,
        reader: &mut Reader<BufReader<File>>,
        registry: &mut TypeRegistry,
        name: String,
        is_local: bool,
    ) -> Result<ComplexType, ParserError> {
        let mut children = Vec::new();
        let mut buf = Vec::new();
        let mut is_in_compositor = false;
        let mut extends_existing_type = false;
        let mut base_type = None::<String>;
        let mut current_element_name = None::<String>;
        let mut order = OrderIndicator::Sequence;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(s)) => match s.name().as_ref() {
                    b"xs:sequence" | b"xs:all" | b"xs:choice" => {
                        if is_in_compositor {
                            return Err(ParserError::UnexpectedStartOfNode(
                                std::str::from_utf8(s.name().0)
                                    .unwrap_or("Unknown")
                                    .to_owned(),
                            ));
                        }

                        is_in_compositor = true;

                        match s.name().as_ref() {
                            b"xs:all" => order = OrderIndicator::All,
                            b"xs:choice" => order = OrderIndicator::Choice,
                            b"xs:sequence" => order = OrderIndicator::Sequence,
                            _ => (),
                        }
                    }
                    b"xs:element" => {
                        current_element_name = Some(Self::get_attribute_value(&s, "name")?);
                    }
                    b"xs:complexContent" => {
                        if extends_existing_type {
                            return Err(ParserError::UnexpectedStartOfNode(
                                std::str::from_utf8(s.name().0)
                                    .unwrap_or("Unknown")
                                    .to_owned(),
                            ));
                        }

                        extends_existing_type = true;
                    }
                    b"xs:extension" => {
                        if !extends_existing_type {
                            return Err(ParserError::UnexpectedStartOfNode(
                                std::str::from_utf8(s.name().0)
                                    .unwrap_or("Unknown")
                                    .to_owned(),
                            ));
                        }

                        let b_type = Self::get_attribute_value(&s, "base")?;
                        base_type = Some(self.resolve_namespace(b_type)?);
                    }
                    b"xs:simpleType" => {
                        if let Some(name) = &current_element_name {
                            let (s_type, type_name) =
                                self.parse_simple_type(reader, name.clone(), true)?;

                            registry.register_type(s_type);

                            let base_attributes = Self::get_base_attributes(&s)?;

                            let node = Node::new(
                                NodeType::Custom(type_name),
                                name.clone(),
                                base_attributes,
                            );
                            children.push(node);
                        } else {
                            let name = Self::get_attribute_value(&s, "name")
                                .ok()
                                .unwrap_or_else(|| registry.generate_type_name());

                            let (s_type, _) = self.parse_simple_type(reader, name, true)?;

                            registry.register_type(s_type);
                        }
                    }
                    b"xs:complexType" => {
                        if let Some(name) = &current_element_name {
                            let c_type =
                                self.parse_complex_type(reader, registry, name.clone(), true)?;

                            let c_type = CustomTypeDefinition::Complex(c_type);
                            registry.register_type(c_type);

                            let base_attributes = Self::get_base_attributes(&s)?;

                            let node = Node::new(
                                NodeType::Custom(name.clone()),
                                name.clone(),
                                base_attributes,
                            );
                            children.push(node);
                        } else {
                            let name = Self::get_attribute_value(&s, "name")
                                .ok()
                                .unwrap_or_else(|| registry.generate_type_name());

                            let c_type = self.parse_complex_type(reader, registry, name, true)?;
                            let c_type = CustomTypeDefinition::Complex(c_type);

                            registry.register_type(c_type);
                        }
                    }
                    _ => (),
                },
                Ok(Event::Empty(e)) => {
                    if let b"xs:element" = e.name().as_ref() {
                        let name = Self::get_attribute_value(&e, "name")?;
                        let b_type = Self::get_attribute_value(&e, "type")?;
                        let b_type = self.resolve_namespace(b_type)?;

                        let node_type = match Self::base_type_str_to_node_type(b_type.as_str()) {
                            Some(t) => t,
                            None => return Err(ParserError::MissingOrNotSupportedBaseType(b_type)),
                        };
                        let base_attributes = Self::get_base_attributes(&e)?;

                        let node = Node::new(node_type, name, base_attributes);

                        children.push(node);
                    }
                }
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"xs:complexType" => break,
                    b"xs:element" => current_element_name = None,
                    _ => continue,
                },
                Ok(Event::Eof) => return Err(ParserError::UnexpectedEndOfFile),
                Err(_) => return Err(ParserError::UnexpectedError),
                _ => (),
            }

            buf.clear();
        }

        Ok(ComplexType {
            name: name.clone(),
            qualified_name: if is_local {
                None
            } else {
                Some(self.as_qualified_name(name.as_str()))
            },
            base_type,
            children,
            order,
        })
    }

    fn parse_simple_type(
        &self,
        reader: &mut Reader<BufReader<File>>,
        name: String,
        is_local: bool,
    ) -> Result<(CustomTypeDefinition, String), ParserError> {
        let mut base_type = String::new();
        let mut list_type = String::new();
        let mut enumerations = Vec::new();
        let mut pattern = None::<String>;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(s)) => {
                    if let b"xs:restriction" = s.name().as_ref() {
                        base_type = Self::get_attribute_value(&s, "base")?
                    }
                }
                Ok(Event::Empty(e)) => match e.name().as_ref() {
                    b"xs:enumeration" => {
                        let value = Self::get_attribute_value(&e, "value")?;
                        enumerations.push(value);
                    }
                    b"xs:list" => {
                        let l_type = Self::get_attribute_value(&e, "itemType")?;
                        list_type = self.resolve_namespace(l_type)?;
                    }
                    b"xs:pattern" => {
                        let value = Self::get_attribute_value(&e, "value")?;
                        pattern = Some(value);
                    }
                    _ => (),
                },
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"xs:simpleType" => break,
                    _ => continue,
                },
                Ok(Event::Eof) => return Err(ParserError::UnexpectedEndOfFile),
                Err(_) => return Err(ParserError::UnexpectedError),
                _ => (),
            }
        }

        let qualified_name = if is_local {
            None
        } else {
            Some(self.as_qualified_name(name.as_str()))
        };

        let s_type = CustomTypeDefinition::Simple(SimpleType {
            name: name.clone(),
            qualified_name,
            base_type: Self::base_type_str_to_node_type(base_type.as_str()),
            enumeration: if enumerations.is_empty() {
                None
            } else {
                Some(enumerations)
            },
            list_type: Self::base_type_str_to_node_type(list_type.as_str()),
            pattern,
        });

        buf.clear();

        Ok((s_type, name))
    }

    fn base_type_str_to_node_type(base_type: &str) -> Option<NodeType> {
        match base_type {
            "xs:base64Binary" => Some(NodeType::Standard(NodeBaseType::Base64Binary)),
            "xs:boolean" => Some(NodeType::Standard(NodeBaseType::Boolean)),
            "xs:date" => Some(NodeType::Standard(NodeBaseType::Date)),
            "xs:dateTime" => Some(NodeType::Standard(NodeBaseType::DateTime)),
            "xs:decimal" => Some(NodeType::Standard(NodeBaseType::Decimal)),
            "xs:double" => Some(NodeType::Standard(NodeBaseType::Double)),
            "xs:float" => Some(NodeType::Standard(NodeBaseType::Float)),
            "xs:hexBinary" => Some(NodeType::Standard(NodeBaseType::HexBinary)),
            "xs:string" => Some(NodeType::Standard(NodeBaseType::String)),
            "xs:time" => Some(NodeType::Standard(NodeBaseType::Time)),
            "" => None,
            _ => Some(NodeType::Custom((*base_type).to_owned())),
        }
    }

    fn get_attribute_value(node: &BytesStart, name: &str) -> Result<String, ParserError> {
        match node
            .attributes()
            .find(|a| a.as_ref().is_ok_and(|v| v.key.0 == name.as_bytes()))
        {
            Some(r) => match r {
                Ok(a) => match a.value {
                    Cow::Borrowed(v) => String::from_utf8(v.to_vec()).map_err(|e| {
                        ParserError::MalformedAttribute(
                            String::from(name),
                            Some(format!("{:?}", e)),
                        )
                    }),
                    Cow::Owned(v) => String::from_utf8(v).map_err(|e| {
                        ParserError::MalformedAttribute(
                            String::from(name),
                            Some(format!("{:?}", e)),
                        )
                    }),
                },
                Err(e) => Err(ParserError::MalformedAttribute(
                    String::from(name),
                    Some(format!("{:?}", e)),
                )),
            },
            None => Err(ParserError::MissingAttribute(String::from(name))),
        }
    }

    fn get_base_attributes(node: &BytesStart) -> Result<BaseAttributes, ParserError> {
        let min_occurs = Self::get_occurance_value(node, "minOccurs")?;
        let max_occurs = Self::get_occurance_value(node, "maxOccurs")?;

        Ok(BaseAttributes {
            min_occurs,
            max_occurs,
        })
    }

    fn get_occurance_value(node: &BytesStart, name: &str) -> Result<Option<i64>, ParserError> {
        let occurance = match Self::get_attribute_value(node, name) {
            Ok(v) => Some(v),
            Err(ParserError::MissingAttribute(_)) => None,
            Err(e) => return Err(e),
        };

        let occurance = match occurance {
            Some(v) => {
                if v == *"unbounded" {
                    Some(UNBOUNDED_OCCURANCE)
                } else {
                    match v.parse::<i64>() {
                        Ok(m) => Some(m),
                        Err(e) => {
                            return Err(ParserError::MalformedAttribute(
                                name.to_owned(),
                                Some(format!("{:?}", e)),
                            ));
                        }
                    }
                }
            }
            None => None,
        };

        Ok(occurance)
    }

    fn lookup_namespace(&self, alias: &String) -> Option<&String> {
        self.namespace_aliases.get(alias)
    }

    fn as_qualified_name(&self, name: &str) -> String {
        let mut qualified_name = self.current_namespace.clone().unwrap_or_default();
        if !qualified_name.ends_with('/') {
            qualified_name.push('/');
        }
        qualified_name.push_str(name);

        qualified_name
    }

    fn resolve_namespace(&self, b_type: String) -> Result<String, ParserError> {
        if b_type.starts_with("xs:") {
            return Ok(b_type);
        }

        let colon_position = b_type.find(':');

        match colon_position {
            Some(position) => {
                let alias = b_type[..position].to_string();

                match self.lookup_namespace(&alias) {
                    Some(n) => Ok(n.clone()),
                    None => Err(ParserError::FailedToResolveNamespace(alias)),
                }
            }
            None => Ok(self.as_qualified_name(b_type.as_str())),
        }
    }

    fn extract_schema_namespace_aliases(&mut self, s: &BytesStart<'_>) -> Option<ParserError> {
        let prefix = b"xmlns:";

        for attr in s.attributes().filter(|a| {
            a.as_ref()
                .is_ok_and(|a| a.key.0.starts_with(prefix) && a.key.0 != b"xmlns:xs")
        }) {
            match attr {
                Ok(a) => {
                    let alias = &a.key.0[prefix.len()..];
                    let alias = match std::str::from_utf8(alias) {
                        Ok(v) => String::from(v),
                        Err(e) => {
                            return Some(ParserError::MalformedAttribute(
                                "unknown".to_owned(),
                                Some(format!("{:?}", e)),
                            ))
                        }
                    };

                    let value = match a.value {
                        Cow::Borrowed(v) => std::str::from_utf8(v).map(|s| s.to_owned()).ok(),
                        Cow::Owned(v) => String::from_utf8(v).ok(),
                    };

                    let value = match value {
                        Some(v) => v,
                        None => return Some(ParserError::MalformedAttribute(alias, None)),
                    };

                    self.namespace_aliases.insert(alias, value);
                }
                Err(e) => return Some(ParserError::MalformedNamespaceAttribute(e.to_string())),
            }
        }

        None
    }
}
