use std::{fs::File, io::BufReader};

use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};

use crate::type_registry::TypeRegistry;

use super::{
    annotations::AnnotationsParser,
    helper::XmlParserHelper,
    types::{EnumerationVariant, NodeType, ParserError, SimpleType, UnionVariant},
    xml::XmlParser,
};

/// Parser for xs:simpleType elements
///
/// Parses into a [SimpleType] struct
/// Supports the following elements:
/// - xs:restriction (partially)
/// - xs:enumeration
/// - xs:annotation
/// - xs:list
/// - xs:pattern (partially)
/// - xs:union
pub struct SimpleTypeParser;

impl SimpleTypeParser {
    /// Parses a xs:simpleType element into a [SimpleType] struct
    ///
    /// # Errors
    ///
    /// Returns an error if the element is malformed
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use quick_xml::Reader;
    ///
    /// use xsd_parser::parser::{SimpleTypeParser, types::NodeType};
    /// use xsd_parser::type_registry::TypeRegistry;
    /// use xsd_parser::parser::xml::XmlParser;
    ///
    /// let mut reader = Reader::from_file("./tests/parser/xsd/simple_type.xsd").unwrap();
    /// let mut buf = Vec::new();
    /// let mut registry = TypeRegistry::new();
    /// let xml_parser = XmlParser::default();
    ///
    /// reader.read_to_end(&mut buf).unwrap();
    ///
    /// let simple_type = SimpleTypeParser::parse(&mut reader, &mut registry, &xml_parser, "SimpleType".to_owned(), None).unwrap();
    ///
    /// assert_eq!(simple_type.name, "SimpleType");
    /// assert_eq!(simple_type.qualified_name, "SimpleType");
    /// assert_eq!(simple_type.base_type, NodeType::Standard(NodeBaseType::String));
    /// assert_eq!(simple_type.enumeration, Some(vec![
    ///   EnumerationVariant {
    ///     name: "Value1".to_owned(),
    ///     documentations: vec![],
    ///   },
    ///   EnumerationVariant {
    ///     name: "Value2".to_owned(),
    ///     documentations: vec![],
    ///   },
    /// ]));
    /// assert_eq!(simple_type.list_type, None);
    /// assert_eq!(simple_type.pattern, None);
    /// assert_eq!(simple_type.variants, None);
    /// assert_eq!(simple_type.documentations, vec![]);
    /// ```
    pub fn parse(
        reader: &mut Reader<BufReader<File>>,
        registry: &mut TypeRegistry,
        xml_parser: &XmlParser,
        name: String,
        qualified_parent: Option<String>,
    ) -> Result<SimpleType, ParserError> {
        let mut base_type = String::new();
        let mut list_type = String::new();
        let mut annotations = Vec::new();
        let mut enumerations = Vec::new();
        let mut pattern = None::<String>;
        let mut variants = None::<Vec<UnionVariant>>;
        let mut buf = Vec::new();
        let mut current_enum_variant = None::<EnumerationVariant>;

        let qualified_name = qualified_parent.map_or_else(
            || xml_parser.as_qualified_name(name.as_str()),
            |v| format!("{v}.{name}"),
        );

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(s)) => match s.name().as_ref() {
                    b"xs:restriction" => {
                        base_type = XmlParserHelper::get_attribute_value(&s, "base")?;
                    }
                    b"xs:union" => {
                        if variants.is_some() {
                            return Err(ParserError::UnexpectedStartOfNode("xs:union".to_owned()));
                        }

                        let types = Self::parse_union_local_variants(
                            &s,
                            reader,
                            registry,
                            xml_parser,
                            &name,
                            &qualified_name,
                        )?;
                        variants = Some(types);
                    }
                    b"xs:enumeration" => {
                        if current_enum_variant.is_some() {
                            return Err(ParserError::UnexpectedStartOfNode(
                                "xs:enumeration".to_owned(),
                            ));
                        }

                        let value = XmlParserHelper::get_attribute_value(&s, "value")?;

                        current_enum_variant = Some(EnumerationVariant {
                            name: value,
                            documentations: vec![],
                        });
                    }
                    b"xs:annotation" => {
                        let mut values = AnnotationsParser::parse(reader)?;

                        if let Some(variant) = current_enum_variant.as_mut() {
                            variant.documentations.append(&mut values);
                        } else {
                            annotations.append(&mut values);
                        }
                    }
                    _ => (),
                },
                Ok(Event::Empty(e)) => match e.name().as_ref() {
                    b"xs:restriction" => {
                        base_type = XmlParserHelper::get_attribute_value(&e, "base")?;
                        break;
                    }
                    b"xs:enumeration" => {
                        let value = XmlParserHelper::get_attribute_value(&e, "value")?;
                        enumerations.push(EnumerationVariant {
                            name: value,
                            documentations: vec![],
                        });
                    }
                    b"xs:list" => {
                        let l_type = XmlParserHelper::get_attribute_value(&e, "itemType")?;
                        list_type = xml_parser.resolve_namespace(l_type)?;
                    }
                    b"xs:pattern" => {
                        let value = XmlParserHelper::get_attribute_value(&e, "value")?;
                        pattern = Some(value);
                    }
                    b"xs:union" => {
                        if variants.is_some() {
                            return Err(ParserError::UnexpectedStartOfNode("xs:union".to_owned()));
                        }

                        let types = Self::get_union_member_types(&e, xml_parser)?;
                        variants = Some(types);
                    }
                    _ => (),
                },
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"xs:enumeration" => {
                        if current_enum_variant.is_none() {
                            return Err(ParserError::UnexpectedError); // TODO: Add better error
                        }

                        let variant = current_enum_variant.unwrap();
                        enumerations.push(variant);

                        current_enum_variant = None;
                    }
                    b"xs:simpleType" => break,
                    _ => continue,
                },
                Ok(Event::Eof) => return Err(ParserError::UnexpectedEndOfFile),
                Err(e) => {
                    println!("{e}");

                    return Err(ParserError::UnexpectedError);
                }
                _ => (),
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }

        let base_type = xml_parser.resolve_namespace(base_type)?;

        let s_type = SimpleType {
            name,
            qualified_name,
            base_type: XmlParserHelper::base_type_str_to_node_type(base_type.as_str()),
            enumeration: if enumerations.is_empty() {
                None
            } else {
                Some(enumerations)
            },
            list_type: XmlParserHelper::base_type_str_to_node_type(list_type.as_str()),
            pattern,
            variants,
            documentations: annotations,
        };

        buf.clear();

        Ok(s_type)
    }

    fn parse_union_local_variants(
        node: &BytesStart,
        reader: &mut Reader<BufReader<File>>,
        registry: &mut TypeRegistry,
        xml_parser: &XmlParser,
        name: &String,
        qualified_parent: &str,
    ) -> Result<Vec<UnionVariant>, ParserError> {
        let mut types = match Self::get_union_member_types(node, xml_parser) {
            Ok(v) => v,
            Err(ParserError::MissingAttribute(_)) => Vec::new(),
            Err(e) => return Err(e),
        };
        let mut variant_count: usize = types.len() + 1;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(s)) if s.name().as_ref() == b"xs:simpleType" => {
                    let variant_name = format!("{name}Variant{variant_count}");

                    let s_type = Self::parse(
                        reader,
                        registry,
                        xml_parser,
                        variant_name,
                        Some(qualified_parent.to_owned()),
                    )?;

                    registry.register_type(s_type.clone().into());

                    types.push(UnionVariant::Simple(s_type));

                    variant_count += 1;
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"xs:union" => break,
                Ok(_) => (),
                Err(_) => return Err(ParserError::UnexpectedError),
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }

        Ok(types)
    }

    fn get_union_member_types(
        node: &BytesStart,
        xml_parser: &XmlParser,
    ) -> Result<Vec<UnionVariant>, ParserError> {
        let member_types = XmlParserHelper::get_attribute_value(node, "memberTypes")?;

        member_types
            .split(' ')
            .filter_map(XmlParserHelper::base_type_str_to_node_type)
            .map(|t| match t {
                NodeType::Standard(t) => Ok(UnionVariant::Standard(t)),
                NodeType::Custom(n) => Ok(UnionVariant::Named(xml_parser.resolve_namespace(n)?)),
            })
            .collect::<Vec<Result<UnionVariant, ParserError>>>()
            .into_iter()
            .collect()
    }
}
