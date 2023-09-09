use std::{borrow::Cow, collections::HashMap, fs::File, io::BufReader, path::Path};

use quick_xml::{events::BytesStart, events::Event, Reader};

use super::{
    annotations::AnnotationsParser,
    complex_type::ComplexTypeParser,
    helper::XmlParserHelper,
    simple_type::SimpleTypeParser,
    types::{BaseAttributes, CustomTypeDefinition, Node, NodeType, ParsedData, ParserError},
};
use crate::type_registry::TypeRegistry;

#[derive(Default)]
pub(crate) struct XmlParser {
    pub(crate) current_namespace: Option<String>,
    pub(crate) namespace_aliases: HashMap<String, String>,
}

impl XmlParser {
    pub(crate) fn parse_file<P: AsRef<Path>>(
        &mut self,
        path: P,
        registry: &mut TypeRegistry,
    ) -> Result<ParsedData, ParserError> {
        let Ok(mut reader) = Reader::from_file(path) else {
            return Err(ParserError::UnableToReadFile);
        };

        self.parse_nodes(&mut reader, registry)
    }

    pub(crate) fn parse_files<P: AsRef<Path>>(
        &mut self,
        paths: &Vec<P>,
        registry: &mut TypeRegistry,
    ) -> Result<ParsedData, ParserError> {
        let mut nodes = Vec::new();
        let mut documentations = Vec::new();

        for path in paths {
            let Ok(mut reader) = Reader::from_file(path) else { return Err(ParserError::UnableToReadFile) };

            self.current_namespace = None;
            self.namespace_aliases.clear();

            let file_nodes = self.parse_nodes(&mut reader, registry)?;
            nodes.extend(file_nodes.nodes);
            documentations.extend(file_nodes.documentations);
        }

        Ok(ParsedData {
            nodes,
            documentations,
        })
    }

    fn parse_nodes(
        &mut self,
        reader: &mut Reader<BufReader<File>>,
        registry: &mut TypeRegistry,
    ) -> Result<ParsedData, ParserError> {
        let mut nodes = Vec::new();
        let mut documentations = Vec::new();
        let mut buf = Vec::new();

        let mut current_element = None::<(String, BaseAttributes)>;

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Eof) => break,
                Ok(Event::Start(s)) => {
                    match s.name().as_ref() {
                        b"xs:schema" => {
                            self.current_namespace =
                                XmlParserHelper::get_attribute_value(&s, "targetNamespace").ok();

                            self.extract_schema_namespace_aliases(&s);
                        }
                        b"xs:element" => {
                            let name = XmlParserHelper::get_attribute_value(&s, "name")?;
                            let base_attributes = XmlParserHelper::get_base_attributes(&s)?;

                            current_element = Some((name, base_attributes));
                        }
                        b"xs:complexType" => {
                            if let Some((name, base_attributes)) = &current_element {
                                let c_type = ComplexTypeParser::parse(
                                    reader,
                                    registry,
                                    self,
                                    name.clone(),
                                    None,
                                )?;

                                let node_type = NodeType::Custom(c_type.qualified_name.clone());
                                let c_type = CustomTypeDefinition::Complex(c_type);
                                registry.register_type(c_type);

                                let node =
                                    Node::new(node_type, name.clone(), (*base_attributes).clone());
                                nodes.push(node);
                            } else {
                                let name = XmlParserHelper::get_attribute_value(&s, "name")
                                    .ok()
                                    .unwrap_or_else(|| registry.generate_type_name());

                                let c_type =
                                    ComplexTypeParser::parse(reader, registry, self, name, None)?;

                                let c_type = CustomTypeDefinition::Complex(c_type);

                                registry.register_type(c_type);
                            }
                        }
                        b"xs:simpleType" => {
                            if let Some((name, base_attributes)) = &current_element {
                                let s_type = SimpleTypeParser::parse(
                                    reader,
                                    registry,
                                    self,
                                    name.clone(),
                                    None,
                                )?;

                                let node_type = NodeType::Custom(s_type.qualified_name.clone());
                                registry.register_type(s_type.into());

                                let node =
                                    Node::new(node_type, name.clone(), (*base_attributes).clone());
                                nodes.push(node);
                            } else {
                                let name = XmlParserHelper::get_attribute_value(&s, "name")
                                    .ok()
                                    .unwrap_or_else(|| registry.generate_type_name());

                                let s_type =
                                    SimpleTypeParser::parse(reader, registry, self, name, None)?;

                                registry.register_type(s_type.into());
                            }
                        }
                        b"xs:annotation" => {
                            let mut values = AnnotationsParser::parse(reader)?;
                            documentations.append(&mut values);
                        }
                        _ => (),
                    }
                    //
                }
                Ok(Event::End(e)) => {
                    if let b"xs:element" = e.name().as_ref() {
                        current_element = None;
                    }
                }
                Ok(Event::Empty(e)) => {
                    if let b"xs:element" = e.name().as_ref() {
                        let name = XmlParserHelper::get_attribute_value(&e, "name")?;
                        let b_type = XmlParserHelper::get_attribute_value(&e, "type")?;
                        let b_type = self.resolve_namespace(b_type)?;
                        let Some(node_type) = XmlParserHelper::base_type_str_to_node_type(b_type.as_str()) else {
                            return Err(ParserError::MissingOrNotSupportedBaseType(b_type))
                        };

                        let base_attributes = XmlParserHelper::get_base_attributes(&e)?;
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

        Ok(ParsedData {
            nodes,
            documentations,
        })
    }

    #[inline]
    fn lookup_namespace(&self, alias: &String) -> Option<&String> {
        self.namespace_aliases.get(alias)
    }

    #[inline]
    pub(crate) fn as_qualified_name(&self, name: &str) -> String {
        let mut qualified_name = match self.current_namespace.clone() {
            Some(mut current_namespace) => {
                if !current_namespace.ends_with('/') {
                    current_namespace.push('/');
                }

                current_namespace
            }
            None => String::new(),
        };
        qualified_name.push_str(name);

        qualified_name
    }

    pub(crate) fn resolve_namespace(&self, b_type: String) -> Result<String, ParserError> {
        if b_type.is_empty() || b_type.starts_with("xs:") {
            return Ok(b_type);
        }

        match b_type.find(':') {
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
                .is_ok_and(|a| a.key.0.starts_with(prefix) && a.key.0 != prefix)
        }) {
            match attr {
                Ok(a) => {
                    let alias = &a.key.0[prefix.len()..];
                    let alias = match std::str::from_utf8(alias) {
                        Ok(v) => String::from(v),
                        Err(e) => {
                            return Some(ParserError::MalformedAttribute(
                                "unknown".to_owned(),
                                Some(format!("{e:?}")),
                            ))
                        }
                    };

                    let value = match a.value {
                        Cow::Borrowed(v) => std::str::from_utf8(v)
                            .map(std::borrow::ToOwned::to_owned)
                            .ok(),
                        Cow::Owned(v) => String::from_utf8(v).ok(),
                    };

                    let Some(value) = value else { return Some(ParserError::MalformedAttribute(alias, None)) };

                    self.namespace_aliases.insert(alias, value);
                }
                Err(e) => return Some(ParserError::MalformedNamespaceAttribute(e.to_string())),
            }
        }

        None
    }
}
