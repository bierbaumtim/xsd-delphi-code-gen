mod class_type;
mod enumeration;
mod helper;
mod type_alias;
mod union_type;

use crate::{
    parser::types::{
        CustomTypeDefinition, NodeType, ParsedData, DEFAULT_OCCURANCE, UNBOUNDED_OCCURANCE,
    },
    type_registry::TypeRegistry,
};

pub use super::{
    dependency_graph::DependencyGraph,
    types::{ClassType, DataType, Enumeration, TypeAlias, UnionType, Variable, XMLSource},
};

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
    pub fn build(data: &ParsedData, registry: &TypeRegistry) -> Self {
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
                    if let Some(lt) = &st.list_type {
                        if let Some(d_type) = helper::list_type_to_data_type(lt, registry) {
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

        let document_variables = Self::get_document_variables(data, registry);

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

    /// Gets the variables of the document.
    /// The document is a special class type that contains the root element of the XML document.
    /// The variables of the document are the root elements of the XML document.
    ///
    /// # Arguments
    /// * `data` - The parsed data.
    /// * `registry` - The type registry.
    /// # Returns
    /// The variables of the document.
    fn get_document_variables(data: &ParsedData, registry: &TypeRegistry) -> Vec<Variable> {
        data.nodes
            .iter()
            .map(|node| {
                let min_occurs = node.base_attributes.min_occurs.unwrap_or(DEFAULT_OCCURANCE);
                let max_occurs = node.base_attributes.max_occurs.unwrap_or(DEFAULT_OCCURANCE);

                Variable {
                    name: node.name.clone(),
                    xml_name: node.name.clone(),
                    required: min_occurs > 0,
                    data_type: match &node.node_type {
                        NodeType::Standard(s) => helper::node_base_type_to_datatype(s),
                        NodeType::Custom(e) => {
                            let c_type = registry.types.get(e);

                            match c_type {
                                Some(c_type) => {
                                    let data_type = match c_type {
                                        CustomTypeDefinition::Simple(s)
                                            if s.enumeration.is_some() =>
                                        {
                                            DataType::Enumeration(s.name.clone())
                                        }
                                        CustomTypeDefinition::Simple(s)
                                            if s.base_type.is_some() || s.list_type.is_some() =>
                                        {
                                            DataType::Alias(s.name.clone())
                                        }
                                        CustomTypeDefinition::Simple(s) if s.variants.is_some() => {
                                            DataType::Union(s.name.clone())
                                        }
                                        _ => DataType::Custom(c_type.get_name()),
                                    };

                                    if max_occurs == UNBOUNDED_OCCURANCE
                                        || (min_occurs != max_occurs
                                            && max_occurs > DEFAULT_OCCURANCE)
                                    {
                                        DataType::List(Box::new(data_type))
                                    } else if min_occurs == max_occurs
                                        && max_occurs > DEFAULT_OCCURANCE
                                    {
                                        let size = usize::try_from(max_occurs).unwrap();

                                        DataType::FixedSizeList(Box::new(data_type), size)
                                    } else {
                                        data_type
                                    }
                                }
                                None => todo!(),
                            }
                        }
                    },
                    requires_free: match &node.node_type {
                        NodeType::Standard(_) => false,
                        NodeType::Custom(c) => registry.types.get(c).map_or(false, |t| match t {
                            CustomTypeDefinition::Simple(s) => s.list_type.is_some(),
                            CustomTypeDefinition::Complex(_) => true,
                        }),
                    },
                    default_value: None,
                    is_const: false,
                    source: XMLSource::Element,
                }
            })
            .collect::<Vec<Variable>>()
    }
}
