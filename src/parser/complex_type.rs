use std::{fs::File, io::BufReader};

use quick_xml::{events::Event, Reader};

use crate::type_registry::TypeRegistry;

use super::{
    annotations::AnnotationsParser,
    helper::XmlParserHelper,
    simple_type::SimpleTypeParser,
    types::{ComplexType, CustomTypeDefinition, Node, NodeType, OrderIndicator, ParserError},
    xml::XmlParser,
};

pub(crate) struct ComplexTypeParser;

impl ComplexTypeParser {
    pub(crate) fn parse(
        reader: &mut Reader<BufReader<File>>,
        registry: &mut TypeRegistry,
        xml_parser: &XmlParser,
        name: String,
        qualified_parent: Option<String>,
    ) -> Result<ComplexType, ParserError> {
        let mut children = Vec::new();
        let mut buf = Vec::new();
        let mut is_in_compositor = false;
        let mut extends_existing_type = false;
        let mut base_type = None::<String>;
        let mut annotations = Vec::new();
        let mut current_element_name = None::<String>;
        let mut order = OrderIndicator::Sequence;

        let qualified_name = qualified_parent.map_or_else(
            || xml_parser.as_qualified_name(name.as_str()),
            |v| format!("{}.{}", v, name),
        );

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
                        current_element_name =
                            Some(XmlParserHelper::get_attribute_value(&s, "name")?);
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

                        let b_type = XmlParserHelper::get_attribute_value(&s, "base")?;
                        base_type = Some(xml_parser.resolve_namespace(b_type)?);
                    }
                    b"xs:simpleType" => {
                        if let Some(name) = &current_element_name {
                            let (s_type, type_name) = SimpleTypeParser::parse(
                                reader,
                                registry,
                                xml_parser,
                                name.clone(),
                                Some(qualified_name.clone()),
                            )?;

                            registry.register_type(s_type.into());

                            let base_attributes = XmlParserHelper::get_base_attributes(&s)?;

                            let node = Node::new(
                                NodeType::Custom(type_name),
                                name.clone(),
                                base_attributes,
                            );
                            children.push(node);
                        } else {
                            let name = XmlParserHelper::get_attribute_value(&s, "name")
                                .ok()
                                .unwrap_or_else(|| registry.generate_type_name());

                            let (s_type, _) = SimpleTypeParser::parse(
                                reader,
                                registry,
                                xml_parser,
                                name,
                                Some(qualified_name.clone()),
                            )?;

                            registry.register_type(s_type.into());
                        }
                    }
                    b"xs:complexType" => {
                        if let Some(name) = &current_element_name {
                            let c_type = Self::parse(
                                reader,
                                registry,
                                xml_parser,
                                name.clone(),
                                Some(qualified_name.clone()),
                            )?;

                            let c_type = CustomTypeDefinition::Complex(c_type);
                            registry.register_type(c_type);

                            let base_attributes = XmlParserHelper::get_base_attributes(&s)?;

                            let node = Node::new(
                                NodeType::Custom(name.clone()),
                                name.clone(),
                                base_attributes,
                            );
                            children.push(node);
                        } else {
                            let name = XmlParserHelper::get_attribute_value(&s, "name")
                                .ok()
                                .unwrap_or_else(|| registry.generate_type_name());

                            let c_type = Self::parse(
                                reader,
                                registry,
                                xml_parser,
                                name,
                                Some(qualified_name.clone()),
                            )?;
                            let c_type = CustomTypeDefinition::Complex(c_type);

                            registry.register_type(c_type);
                        }
                    }
                    b"xs:annotation" if current_element_name.is_none() => {
                        let mut values = AnnotationsParser::parse(reader)?;
                        annotations.append(&mut values);
                    }
                    _ => (),
                },
                Ok(Event::Empty(e)) => {
                    if let b"xs:element" = e.name().as_ref() {
                        let name = XmlParserHelper::get_attribute_value(&e, "name")?;
                        let b_type = XmlParserHelper::get_attribute_value(&e, "type")?;
                        let b_type = xml_parser.resolve_namespace(b_type)?;

                        let node_type =
                            match XmlParserHelper::base_type_str_to_node_type(b_type.as_str()) {
                                Some(t) => t,
                                None => {
                                    return Err(ParserError::MissingOrNotSupportedBaseType(b_type))
                                }
                            };
                        let base_attributes = XmlParserHelper::get_base_attributes(&e)?;

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
            name,
            qualified_name,
            base_type,
            children,
            order,
            documentations: annotations,
        })
    }
}