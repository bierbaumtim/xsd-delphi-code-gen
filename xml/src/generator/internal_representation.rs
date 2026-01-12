mod class_type;
mod enumeration;
mod helper;
mod type_alias;
mod union_type;

use crate::parser::types::{CustomTypeDefinition, OrderIndicator, ParsedData};

use self::class_type::collect_variables;

pub use super::types::{
    ClassType, DataType, Enumeration, TypeAlias, UnionType, Variable, XMLSource,
};
pub use genphi_core::{dependency_graph::DependencyGraph, type_registry::TypeRegistry};

/// The name of the document class type.
pub const DOCUMENT_NAME: &str = "Document";

/// This is the internal representation of the XML Schema.
/// It contains the classes, enumerations, type aliases and union types.
/// It also contains the document class type.
/// The document class type is a special class type that contains the root element of the XML document.
/// The root element is the element that is defined in the XML Schema as the root element of the XML document.
///
/// # Fields
/// * `document` - The document class type.
/// * `classes` - The class types.
/// * `types_aliases` - The type aliases.
/// * `enumerations` - The enumerations.
/// * `union_types` - The union types.
///
/// # Examples
///
/// ```rust
/// use generator::internal_representation::InternalRepresentation;
/// use generator::parser::types::ParsedData;
/// use generator::parser::xml::XmlParser;
/// use generator::type_registry::TypeRegistry;
///
/// let xml = r#"
/// <?xml version="1.0" encoding="utf-8"?>
/// <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
///     <xs:element name="root">
///         <xs:complexType>
///             <xs:sequence>
///                 <xs:element name="first" type="xs:string"/>
///                 <xs:element name="second" type="xs:int"/>
///                 <xs:element name="third" type="xs:string"/>
///             </xs:sequence>
///         </xs:complexType>
///     </xs:element>
/// </xs:schema>
/// "#;
///
/// let mut parser = XmlParser::default();
/// let mut type_registry = TypeRegistry::new();
///
/// let data: ParsedData = parser.parse(xml, &mut type_registry).unwrap();
///
/// let ir = InternalRepresentation::build(&data, &type_registry);
/// ```
#[derive(Debug)]
pub struct InternalRepresentation {
    pub document: ClassType,
    pub classes: Vec<ClassType>,
    pub types_aliases: Vec<TypeAlias>,
    pub enumerations: Vec<Enumeration>,
    pub union_types: Vec<UnionType>,
}

impl InternalRepresentation {
    /// Builds the internal representation from the parsed data and the type registry.
    /// It also takes care of the dependencies between the types.
    ///
    /// # Arguments
    ///
    /// * `data` - The parsed data.
    /// * `registry` - The type registry.
    ///
    /// # Returns
    ///
    /// The internal representation.
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
    ///     <xs:element name="root">
    ///         <xs:complexType>
    ///             <xs:sequence>
    ///                 <xs:element name="first" type="xs:string"/>
    ///                 <xs:element name="second" type="xs:int"/>
    ///                 <xs:element name="third" type="xs:string"/>
    ///             </xs:sequence>
    ///         </xs:complexType>
    ///     </xs:element>
    /// </xs:schema>
    /// "#;
    ///
    /// let mut parser = XmlParser::default();
    /// let mut type_registry = TypeRegistry::new();
    ///
    /// let data: ParsedData = parser.parse(xml, &mut type_registry).unwrap();
    ///
    /// let ir = InternalRepresentation::build(&data, &type_registry);
    /// ```
    pub fn build(data: &ParsedData, registry: &TypeRegistry<CustomTypeDefinition>) -> Self {
        let mut classes_dep_graph = DependencyGraph::<String, ClassType>::new();
        let mut aliases_dep_graph = DependencyGraph::<String, TypeAlias>::new();
        let mut union_types_dep_graph = DependencyGraph::<String, UnionType>::new();

        let mut enumerations = Vec::new();

        for c_type in registry.types.values() {
            match c_type {
                CustomTypeDefinition::Simple(st) if st.enumeration.is_some() => {
                    let enumeration = enumeration::build_enumeration_ir(st);

                    enumerations.push(enumeration);
                }
                CustomTypeDefinition::Simple(st) if st.base_type.is_some() => {
                    let alias = type_alias::build_type_alias_ir(st);

                    aliases_dep_graph.push(alias);
                }
                CustomTypeDefinition::Simple(st) if st.list_type.is_some() => {
                    if let Some(lt) = &st.list_type
                        && let Some(d_type) = helper::list_type_to_data_type(lt, registry) {
                            let type_alias = TypeAlias {
                                name: st.name.clone(),
                                qualified_name: st.qualified_name.clone(),
                                for_type: DataType::InlineList(Box::new(d_type)),
                                pattern: None,
                                documentations: st.documentations.clone(),
                            };

                            aliases_dep_graph.push(type_alias);
                        }
                }
                CustomTypeDefinition::Simple(st) if st.variants.is_some() => {
                    let union_type = union_type::build_union_type_ir(st, registry);

                    union_types_dep_graph.push(union_type);
                }
                CustomTypeDefinition::Simple(_) => (),
                CustomTypeDefinition::Complex(ct) => {
                    let class_type = class_type::build_class_type_ir(ct, registry);

                    classes_dep_graph.push(class_type);
                }
            }
        }

        let document_variables =
            collect_variables(&data.nodes, registry, &OrderIndicator::Sequence);

        let document_type = ClassType {
            super_type: None,
            name: String::from(DOCUMENT_NAME),
            qualified_name: String::from(DOCUMENT_NAME),
            variables: document_variables,
            documentations: vec![],
        };

        classes_dep_graph.push(document_type.clone());

        Self {
            document: document_type,
            classes: classes_dep_graph.get_sorted_elements(),
            types_aliases: aliases_dep_graph.get_sorted_elements(),
            union_types: union_types_dep_graph.get_sorted_elements(),
            enumerations,
        }
    }
}
