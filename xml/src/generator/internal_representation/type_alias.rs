use crate::{
    generator::types::{DataType, TypeAlias},
    parser::types::{NodeType, SimpleType},
};

/// Builds the internal representation for a type alias.
///
/// # Arguments
///
/// * `st` - The simple type definition of the type alias. Must have `base_type` set to Some.
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
    // base_type is guaranteed to be Some by caller's is_some() check
    let Some(base_type) = st.base_type.as_ref() else {
        // This should never happen as caller checks is_some() first
        // Using a safe default to avoid panic in production
        return TypeAlias {
            name: st.name.clone(),
            qualified_name: st.qualified_name.clone(),
            pattern: st.pattern.clone(),
            for_type: DataType::String,
            documentations: st.documentations.clone(),
        };
    };
    
    let for_type = match base_type {
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
