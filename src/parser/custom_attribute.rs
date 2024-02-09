use std::{fs::File, io::BufReader};

use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};

use crate::type_registry::TypeRegistry;

use super::{
    annotations::AnnotationsParser,
    helper::XmlParserHelper,
    simple_type::SimpleTypeParser,
    types::{CustomAttribute, NodeType, ParserError},
    xml::XmlParser,
};

pub struct CustomAttributeParser;

impl CustomAttributeParser {
    pub fn parse(
        reader: &mut Reader<BufReader<File>>,
        registry: &mut TypeRegistry,
        xml_parser: &XmlParser,
        qualified_parent: Option<String>,
        start: &BytesStart<'_>,
        has_content: bool,
    ) -> Result<CustomAttribute, ParserError> {
        let mut documentations = Vec::new();

        let name = XmlParserHelper::get_attribute_value(start, "name")?;

        let qualified_name = qualified_parent.map_or_else(
            || xml_parser.as_qualified_name(name.as_str()),
            |v| format!("{v}.{name}"),
        );

        let default_value = match XmlParserHelper::get_attribute_value(start, "default") {
            Ok(v) => Some(v),
            Err(ParserError::MissingAttribute(_)) => None,
            Err(e) => return Err(e),
        };

        let fixed_value = match XmlParserHelper::get_attribute_value(start, "fixed") {
            Ok(v) => Some(v),
            Err(ParserError::MissingAttribute(_)) => None,
            Err(e) => return Err(e),
        };

        let required = match XmlParserHelper::get_attribute_value(start, "use") {
            Ok(v) => v == "required",
            Err(ParserError::MissingAttribute(_)) => false,
            Err(e) => return Err(e),
        };

        let mut node_type = None::<NodeType>;

        if has_content {
            let mut buf = Vec::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(e)) => match e.name().as_ref() {
                        b"xs:annotation" => {
                            let mut values = AnnotationsParser::parse(reader)?;
                            documentations.append(&mut values);
                        }
                        b"xs:simpleType" => {
                            let s_type = SimpleTypeParser::parse(
                                reader,
                                registry,
                                xml_parser,
                                name.clone(),
                                Some(qualified_name.clone()),
                            )?;

                            registry.register_type(s_type.into());

                            node_type = Some(NodeType::Custom(qualified_name.clone()));
                        }
                        _ => (),
                    },
                    Ok(Event::End(e)) => {
                        if e.name().as_ref() == b"xs:attribute" {
                            break;
                        }
                    }
                    Ok(Event::Eof) => return Err(ParserError::UnexpectedEndOfFile),
                    Err(_) => return Err(ParserError::UnexpectedError),
                    _ => (),
                }

                // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
                buf.clear();
            }
        }

        let node_type = match node_type {
            Some(v) => v,
            None => XmlParserHelper::get_attribute_value(start, "type")
                .and_then(|v| xml_parser.resolve_namespace(v))
                .and_then(|v| {
                    XmlParserHelper::base_type_str_to_node_type(v.as_str())
                        .ok_or(ParserError::MissingOrNotSupportedBaseType(v))
                })?,
        };

        Ok(CustomAttribute {
            name,
            qualified_name,
            documentations,
            base_type: node_type,
            default_value,
            fixed_value,
            required,
        })
    }
}
