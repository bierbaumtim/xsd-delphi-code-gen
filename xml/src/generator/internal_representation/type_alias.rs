use crate::{
    generator::types::{DataType, TypeAlias},
    parser::types::{NodeType, SimpleType},
};

/// Builds the internal representation for a type alias.
///
/// # Arguments
///
/// * `st` - The simple type definition of the type alias.
///
/// # Returns
///
/// The internal representation of the type alias.
///
/// # Examples
///
/// ```
/// use generator::{
///     internal_representation::InternalRepresentation,
///     parser::{
///         types::{
///             CustomTypeDefinition, EnumerationDefinition, EnumerationValueDefinition,
///             ListTypeDefinition, OrderIndicator, ParsedData, SimpleType,
///         },
///         xml::XmlParser,
///     },
///     type_registry::TypeRegistry,
/// };
///
/// let xml = r#"
/// <?xml version="1.0" encoding="utf-8"?>
/// <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
///     <xs:simpleType name="CustomType">
///         <xs:list itemType="xs:string"/>
///     </xs:simpleType>
/// </xs:schema>
/// "#;
///
/// let mut parser = XmlParser::default();
/// let mut type_registry = TypeRegistry::new();
///
/// let data: ParsedData = parser.parse(xml, &mut type_registry).unwrap();
///
/// let ir = InternalRepresentation::build(&data, &type_registry);
///
/// assert_eq!(ir.types_aliases.len(), 1);
/// ```
pub fn build_type_alias_ir(st: &SimpleType) -> TypeAlias {
    let for_type = match st.base_type.as_ref().unwrap() {
        NodeType::Standard(t) => super::helper::node_base_type_to_datatype(t),
        NodeType::Custom(n) => {
            let name = n.split('/').next_back().unwrap_or(n.as_str());

            DataType::Custom(name.to_owned())
        }
    };

    TypeAlias {
        name: st.name.clone(),
        qualified_name: st.qualified_name.clone(),
        pattern: st.pattern.clone(),
        for_type,
        documentations: st.documentations.clone(),
    }
}
