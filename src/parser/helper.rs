use std::borrow::Cow;

use quick_xml::events::BytesStart;

use crate::parser::types::UNBOUNDED_OCCURANCE;

use super::types::{BaseAttributes, NodeBaseType, NodeType, ParserError};

pub(crate) struct XmlParserHelper;

impl XmlParserHelper {
    pub(crate) fn base_type_str_to_node_type(base_type: &str) -> Option<NodeType> {
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
            "xs:anyURI" => Some(NodeType::Standard(NodeBaseType::Uri)),
            "xs:byte" => Some(NodeType::Standard(NodeBaseType::Byte)),
            "xs:short" => Some(NodeType::Standard(NodeBaseType::Short)),
            "xs:nonNegativeInteger"
            | "xs:negativeInteger"
            | "xs:int"
            | "xs:positiveInteger"
            | "xs:nonPositiveInteger" => Some(NodeType::Standard(NodeBaseType::Integer)),
            "xs:long" => Some(NodeType::Standard(NodeBaseType::Long)),
            "xs:unsignedByte" => Some(NodeType::Standard(NodeBaseType::UnsignedByte)),
            "xs:unsignedShort" => Some(NodeType::Standard(NodeBaseType::UnsignedShort)),
            "xs:unsignedInt" => Some(NodeType::Standard(NodeBaseType::UnsignedInteger)),
            "xs:unsignedLong" => Some(NodeType::Standard(NodeBaseType::UnsignedLong)),
            "" => None,
            _ => Some(NodeType::Custom((*base_type).to_owned())),
        }
    }

    pub(crate) fn get_attribute_value(
        node: &BytesStart,
        name: &str,
    ) -> Result<String, ParserError> {
        node.attributes()
            .find(|a| a.as_ref().is_ok_and(|v| v.key.0 == name.as_bytes()))
            .ok_or(ParserError::MissingAttribute(String::from(name)))
            .and_then(|r| {
                r.map_err(|e| {
                    ParserError::MalformedAttribute(String::from(name), Some(format!("{:?}", e)))
                })
            })
            .map(|a| match a.value {
                Cow::Borrowed(v) => String::from_utf8(v.to_vec()).map_err(|e| {
                    ParserError::MalformedAttribute(String::from(name), Some(format!("{:?}", e)))
                }),
                Cow::Owned(v) => String::from_utf8(v).map_err(|e| {
                    ParserError::MalformedAttribute(String::from(name), Some(format!("{:?}", e)))
                }),
            })
            .and_then(|r| r)
    }

    pub(crate) fn get_base_attributes(node: &BytesStart) -> Result<BaseAttributes, ParserError> {
        let min_occurs = Self::get_occurance_value(node, "minOccurs")?;
        let max_occurs = Self::get_occurance_value(node, "maxOccurs")?;

        Ok(BaseAttributes {
            min_occurs,
            max_occurs,
        })
    }

    pub(crate) fn get_occurance_value(
        node: &BytesStart,
        name: &str,
    ) -> Result<Option<i64>, ParserError> {
        let value = Self::get_attribute_value(node, name)
            .map(|v| match v.parse::<i64>() {
                Ok(e) => Ok(e),
                Err(e) => {
                    if v == "unbounded" {
                        Ok(UNBOUNDED_OCCURANCE)
                    } else {
                        Err(ParserError::MalformedAttribute(v, Some(format!("{:?}", e))))
                    }
                }
            })
            .map(|v| v.ok());

        match value {
            Ok(v) => Ok(v),
            Err(ParserError::MissingAttribute(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
