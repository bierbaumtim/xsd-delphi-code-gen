use crate::{
    generator::types::{DataType, UnionType, UnionVariant},
    parser::types::{CustomTypeDefinition, SimpleType},
};
use genphi_core::type_registry::TypeRegistry;

/// Builds the internal representation for a union type.
///
/// # Arguments
///
/// * `st` - The simple type definition of the union type.
///
/// # Returns
///
/// The internal representation of the union type.
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
///         <xs:union memberTypes="xs:string xs:decimal"/>
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
/// assert_eq!(ir.union_types.len(), 1);
/// ```
pub fn build_union_type_ir(
    st: &SimpleType,
    registry: &TypeRegistry<CustomTypeDefinition>,
) -> UnionType {
    // variants is guaranteed to be Some by caller's is_some() check
    let variants = st.variants.as_ref().map_or_else(Vec::new, |variants| {
        variants
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                let d_type = match v {
                    crate::parser::types::UnionVariant::Named(n) => {
                        let Some(CustomTypeDefinition::Simple(st)) = registry.types.get(n) else {
                            return None;
                        };

                        if let Some(lt) = &st.list_type {
                            super::helper::list_type_to_data_type(lt, registry)
                                .map(|d| (DataType::InlineList(Box::new(d)), st.name.clone()))
                        } else if st.enumeration.is_some() {
                            Some((DataType::Enumeration(st.name.clone()), st.name.clone()))
                        } else {
                            Some((DataType::Alias(st.name.clone()), st.name.clone()))
                        }
                    }
                    crate::parser::types::UnionVariant::Simple(st) => {
                        if let Some(lt) = &st.list_type {
                            super::helper::list_type_to_data_type(lt, registry)
                                .map(|d| (DataType::InlineList(Box::new(d)), st.name.clone()))
                        } else if st.enumeration.is_some() {
                            Some((DataType::Enumeration(st.name.clone()), st.name.clone()))
                        } else {
                            Some((DataType::Alias(st.name.clone()), st.name.clone()))
                        }
                    }
                    crate::parser::types::UnionVariant::Standard(t) => Some((
                        super::helper::node_base_type_to_datatype(t),
                        format!("Variant{i}"),
                    )),
                };

                d_type.map(|(dt, name)| UnionVariant {
                    name,
                    data_type: dt,
                })
            })
            .collect::<Vec<UnionVariant>>()
    });
    
    UnionType {
        name: st.name.clone(),
        qualified_name: st.qualified_name.clone(),
        documentations: st.documentations.clone(),
        variants,
    }
}
