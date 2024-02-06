use crate::{
    generator::types::{BinaryEncoding, DataType},
    parser::types::{CustomTypeDefinition, NodeBaseType, NodeType},
    type_registry::TypeRegistry,
};

/// Converts a node base type to a data type.
/// This is used to convert the base types of the nodes to the data types of the variables.
/// The base types are the types that are defined in the XML Schema specification.
/// The data types are the types that are used in the generated code.
///
/// # Arguments
/// * `base_type` - The base type to convert.
/// # Returns
/// The converted data type.
pub const fn node_base_type_to_datatype(base_type: &NodeBaseType) -> DataType {
    match base_type {
        NodeBaseType::Boolean => DataType::Boolean,
        NodeBaseType::DateTime => DataType::DateTime,
        NodeBaseType::Date => DataType::Date,
        NodeBaseType::Decimal | NodeBaseType::Double | NodeBaseType::Float => DataType::Double,
        NodeBaseType::HexBinary => DataType::Binary(BinaryEncoding::Hex),
        NodeBaseType::Base64Binary => DataType::Binary(BinaryEncoding::Base64),
        NodeBaseType::String => DataType::String,
        NodeBaseType::Time => DataType::Time,
        NodeBaseType::Uri => DataType::Uri,
        NodeBaseType::Byte => DataType::ShortInteger,
        NodeBaseType::Short => DataType::SmallInteger,
        NodeBaseType::Integer => DataType::Integer,
        NodeBaseType::Long => DataType::LongInteger,
        NodeBaseType::UnsignedByte => DataType::UnsignedShortInteger,
        NodeBaseType::UnsignedShort => DataType::UnsignedSmallInteger,
        NodeBaseType::UnsignedInteger => DataType::UnsignedInteger,
        NodeBaseType::UnsignedLong => DataType::UnsignedLongInteger,
    }
}

/// Converts a list type to a data type.
/// This is used to convert the list types of the nodes to the data types of the variables.
///
/// # Arguments
/// * `list_type` - The list type to convert.
/// * `registry` - The type registry.
/// # Returns
/// The converted data type.
pub fn list_type_to_data_type(list_type: &NodeType, registry: &TypeRegistry) -> Option<DataType> {
    match list_type {
        NodeType::Standard(s) => Some(super::helper::node_base_type_to_datatype(s)),
        NodeType::Custom(c) => {
            let c_type = registry.types.get(c);

            if let Some(c_type) = c_type {
                return match c_type {
                    CustomTypeDefinition::Simple(s) if s.enumeration.is_some() => {
                        Some(DataType::Enumeration(s.name.clone()))
                    }
                    CustomTypeDefinition::Simple(s) if s.variants.is_some() => {
                        Some(DataType::Union(s.name.clone()))
                    }
                    CustomTypeDefinition::Simple(s) if s.base_type.is_some() => {
                        Some(DataType::Alias(s.name.clone()))
                    }
                    CustomTypeDefinition::Simple(s) if s.list_type.is_some() => None,
                    _ => Some(DataType::Custom(c_type.get_name())),
                };
            }

            None
        }
    }
}
