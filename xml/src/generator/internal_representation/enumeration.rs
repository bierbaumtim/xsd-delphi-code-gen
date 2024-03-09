use crate::{
    generator::types::{Enumeration, EnumerationValue},
    parser::types::SimpleType,
};

/// Builds the internal representation for an enumeration.
///
/// # Arguments
///
/// * `st` - The simple type definition of the enumeration.
///
/// # Returns
///
/// The internal representation of the enumeration.
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
///         <xs:restriction base="xs:string">
///             <xs:enumeration value="First"/>
///             <xs:enumeration value="Second"/>
///             <xs:enumeration value="Third"/>
///         </xs:restriction>
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
/// assert_eq!(ir.enumerations.len(), 1);
/// ```
pub fn build_enumeration_ir(st: &SimpleType) -> Enumeration {
    let values = st
        .enumeration
        .as_ref()
        .unwrap()
        .iter()
        .map(|v| EnumerationValue {
            variant_name: v.name.clone(),
            xml_value: v.name.clone(),
            documentations: v.documentations.clone(),
        })
        .collect::<Vec<EnumerationValue>>();

    Enumeration {
        name: st.name.clone(),
        qualified_name: st.qualified_name.clone(),
        values,
        documentations: st.documentations.clone(),
    }
}
