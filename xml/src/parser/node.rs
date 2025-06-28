use std::fs::File;
use std::io::BufReader;

use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};

use genphi_core::type_registry::TypeRegistry;

use crate::parser::{helper::XmlParserHelper, types::OrderIndicator};

use super::{
    annotations::AnnotationsParser,
    complex_type::ComplexTypeParser,
    simple_type::SimpleTypeParser,
    types::{
        BaseAttributes, CustomTypeDefinition, Node, NodeGroup, NodeType, ParserError, SingleNode,
    },
    xml::XmlParser,
};

pub struct NodeParser;

impl NodeParser {
    pub fn parse_element_with_type_node(
        reader: &mut Reader<BufReader<File>>,
        node_type: NodeType,
        name: String,
        base_attributes: BaseAttributes,
    ) -> Result<Node, ParserError> {
        let mut buf = Vec::new();
        let mut annotations = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(s)) if s.name().as_ref() == b"xs:annotation" => {
                    let mut values = AnnotationsParser::parse(reader)?;
                    annotations.append(&mut values);
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"xs:element" => break,
                Ok(Event::Eof) => return Err(ParserError::UnexpectedEndOfFile),
                Err(_) => return Err(ParserError::UnexpectedError),
                _ => (),
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }

        Ok(Node::Single(SingleNode::new(
            node_type,
            name,
            base_attributes,
            Some(annotations),
        )))
    }

    pub fn parse_node_group(
        reader: &mut Reader<BufReader<File>>,
        registry: &mut TypeRegistry<CustomTypeDefinition>,
        xml_parser: &XmlParser,
        start: &BytesStart,
        qualified_name: String,
    ) -> Result<NodeGroup, ParserError> {
        let mut children: Vec<Node> = Vec::new();
        let mut current_element = None::<(String, BaseAttributes)>;
        let mut buf = Vec::new();

        let order = match start.name().as_ref() {
            b"xs:all" => OrderIndicator::All,
            b"xs:choice" => {
                let base_attributes = XmlParserHelper::get_base_attributes(start)?;
                OrderIndicator::Choice(base_attributes)
            }
            b"xs:sequence" => OrderIndicator::Sequence,
            _ => {
                return Err(ParserError::UnexpectedStartOfNode(
                    std::str::from_utf8(start.name().0)
                        .unwrap_or("Unknown")
                        .to_owned(),
                ));
            }
        };

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(s)) => match s.name().as_ref() {
                    b"xs:element" => {
                        let name = XmlParserHelper::get_attribute_value(&s, "name")?;
                        let base_attributes = XmlParserHelper::get_base_attributes(&s)?;
                        let b_type = XmlParserHelper::get_attribute_value(&s, "type")
                            .and_then(|t| xml_parser.resolve_namespace(t))
                            .and_then(|t| {
                                XmlParserHelper::base_type_str_to_node_type(&t)
                                    .ok_or(ParserError::MissingOrNotSupportedBaseType(t))
                            });

                        match b_type {
                            Ok(node_type) => {
                                current_element = None;

                                let node = NodeParser::parse_element_with_type_node(
                                    reader,
                                    node_type,
                                    name,
                                    base_attributes,
                                )?;

                                children.push(node);
                            }
                            Err(ParserError::MissingAttribute(_)) => {
                                current_element = Some((name, base_attributes));
                            }
                            Err(e) => return Err(e),
                        };
                    }
                    b"xs:complexType" => {
                        if let Some((name, base_attributes)) = &current_element {
                            let c_type = ComplexTypeParser::parse(
                                reader,
                                registry,
                                xml_parser,
                                name.clone(),
                                Some(qualified_name.clone()),
                            )?;

                            let node_type = NodeType::Custom(c_type.qualified_name.clone());
                            let c_type = CustomTypeDefinition::Complex(c_type);
                            registry.register_type(c_type);

                            let node = SingleNode::new(
                                node_type,
                                name.clone(),
                                (*base_attributes).clone(),
                                None,
                            );
                            children.push(Node::Single(node));
                        } else {
                            let name = XmlParserHelper::get_attribute_value(&s, "name")
                                .ok()
                                .unwrap_or_else(|| registry.generate_type_name());

                            let c_type =
                                ComplexTypeParser::parse(reader, registry, xml_parser, name, None)?;

                            let c_type = CustomTypeDefinition::Complex(c_type);

                            registry.register_type(c_type);
                        }
                    }
                    b"xs:simpleType" => {
                        if let Some((name, base_attributes)) = &current_element {
                            let s_type = SimpleTypeParser::parse(
                                reader,
                                registry,
                                xml_parser,
                                name.clone(),
                                Some(qualified_name.clone()),
                            )?;

                            let node_type = NodeType::Custom(s_type.qualified_name.clone());
                            registry.register_type(s_type.into());

                            let node = SingleNode::new(
                                node_type,
                                name.clone(),
                                (*base_attributes).clone(),
                                None,
                            );
                            children.push(Node::Single(node));
                        } else {
                            let name = XmlParserHelper::get_attribute_value(&s, "name")
                                .ok()
                                .unwrap_or_else(|| registry.generate_type_name());

                            let s_type =
                                SimpleTypeParser::parse(reader, registry, xml_parser, name, None)?;

                            registry.register_type(s_type.into());
                        }
                    }
                    _ => (),
                },
                Ok(Event::Empty(e)) if e.name().as_ref() == b"xs:element" => {
                    let name = XmlParserHelper::get_attribute_value(&e, "name")?;
                    let b_type = XmlParserHelper::get_attribute_value(&e, "type")?;
                    let b_type = xml_parser.resolve_namespace(b_type)?;

                    let Some(node_type) =
                        XmlParserHelper::base_type_str_to_node_type(b_type.as_str())
                    else {
                        return Err(ParserError::MissingOrNotSupportedBaseType(b_type));
                    };

                    let base_attributes = XmlParserHelper::get_base_attributes(&e)?;

                    let node = SingleNode::new(node_type, name, base_attributes, None);

                    children.push(Node::Single(node));
                }
                Ok(Event::End(e)) if e.name() == start.name() => break,
                Ok(Event::Eof) => return Err(ParserError::UnexpectedEndOfFile),
                Err(_) => return Err(ParserError::UnexpectedError),
                _ => (),
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }

        Ok(NodeGroup::new(children, order))
    }
}
