use crate::{
    parser::types::{
        CustomTypeDefinition, NodeBaseType, NodeType, OrderIndicator, ParsedData, SimpleType,
        DEFAULT_OCCURANCE, UNBOUNDED_OCCURANCE,
    },
    type_registry::TypeRegistry,
};

use super::{
    dependency_graph::DependencyGraph,
    types::{
        BinaryEncoding, ClassType, DataType, Enumeration, EnumerationValue, TypeAlias, UnionType,
        Variable,
    },
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
                    let enumeration = Self::build_enumeration_ir(st);

                    enumerations.push(enumeration);
                }
                CustomTypeDefinition::Simple(st) if st.base_type.is_some() => {
                    let alias = Self::build_type_alias_ir(st);

                    aliases_dep_graph.push(alias);
                }
                CustomTypeDefinition::Simple(st) if st.list_type.is_some() => {
                    if let Some(lt) = &st.list_type {
                        if let Some(d_type) = Self::list_type_to_data_type(lt, registry) {
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
                    let union_type = Self::build_union_type_ir(st, registry);

                    union_types_dep_graph.push(union_type);
                }
                CustomTypeDefinition::Simple(_) => (),
                CustomTypeDefinition::Complex(ct) => {
                    let class_type = Self::build_class_type_ir(ct, registry);

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
    fn build_enumeration_ir(st: &SimpleType) -> Enumeration {
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
    fn build_type_alias_ir(st: &SimpleType) -> TypeAlias {
        let for_type = match st.base_type.as_ref().unwrap() {
            NodeType::Standard(t) => Self::node_base_type_to_datatype(t),
            NodeType::Custom(n) => {
                let name = n.split('/').last().unwrap_or(n.as_str());

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
    fn build_union_type_ir(st: &SimpleType, registry: &TypeRegistry) -> UnionType {
        UnionType {
            name: st.name.clone(),
            qualified_name: st.qualified_name.clone(),
            documentations: st.documentations.clone(),
            variants: st
                .variants
                .as_ref()
                .unwrap()
                .iter()
                .enumerate()
                .filter_map(|(i, v)| {
                    let d_type = match v {
                        crate::parser::types::UnionVariant::Named(n) => {
                            let Some(CustomTypeDefinition::Simple(st)) = registry.types.get(n)
                            else {
                                return None;
                            };

                            if let Some(lt) = &st.list_type {
                                Self::list_type_to_data_type(lt, registry)
                                    .map(|d| (DataType::InlineList(Box::new(d)), st.name.clone()))
                            } else if st.enumeration.is_some() {
                                Some((DataType::Enumeration(st.name.clone()), st.name.clone()))
                            } else {
                                Some((DataType::Alias(st.name.clone()), st.name.clone()))
                            }
                        }
                        crate::parser::types::UnionVariant::Simple(st) => {
                            if let Some(lt) = &st.list_type {
                                Self::list_type_to_data_type(lt, registry)
                                    .map(|d| (DataType::InlineList(Box::new(d)), st.name.clone()))
                            } else if st.enumeration.is_some() {
                                Some((DataType::Enumeration(st.name.clone()), st.name.clone()))
                            } else {
                                Some((DataType::Alias(st.name.clone()), st.name.clone()))
                            }
                        }
                        crate::parser::types::UnionVariant::Standard(t) => {
                            Some((Self::node_base_type_to_datatype(t), format!("Variant{i}")))
                        }
                    };

                    d_type.map(|(dt, name)| super::types::UnionVariant {
                        name,
                        data_type: dt,
                    })
                })
                .collect::<Vec<super::types::UnionVariant>>(),
        }
    }

    /// Builds the internal representation for a class type.
    ///
    /// # Arguments
    ///
    /// * `ct` - The complex type definition of the class type.
    ///
    /// # Returns
    ///
    /// The internal representation of the class type.
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
    ///     <xs:complexType name="CustomType">
    ///         <xs:sequence>
    ///             <xs:element name="first" type="xs:string"/>
    ///             <xs:element name="second" type="xs:int"/>
    ///             <xs:element name="third" type="xs:string"/>
    ///         </xs:sequence>
    ///     </xs:complexType>
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
    /// assert_eq!(ir.classes.len(), 1);
    /// ```
    fn build_class_type_ir(
        ct: &crate::parser::types::ComplexType,
        registry: &TypeRegistry,
    ) -> ClassType {
        let mut variables = Vec::new();

        for child in &ct.children {
            let min_occurs = match &ct.order {
                OrderIndicator::All => child
                    .base_attributes
                    .min_occurs
                    .unwrap_or(DEFAULT_OCCURANCE)
                    .clamp(0, 1),
                _ => child
                    .base_attributes
                    .min_occurs
                    .unwrap_or(DEFAULT_OCCURANCE),
            };
            let max_occurs = match &ct.order {
                OrderIndicator::All => child
                    .base_attributes
                    .max_occurs
                    .unwrap_or(DEFAULT_OCCURANCE)
                    .clamp(0, 1),
                _ => child
                    .base_attributes
                    .max_occurs
                    .unwrap_or(DEFAULT_OCCURANCE),
            };

            match &child.node_type {
                NodeType::Standard(s) => {
                    let d_type = Self::node_base_type_to_datatype(s);

                    let d_type = if max_occurs == UNBOUNDED_OCCURANCE
                        || (min_occurs != max_occurs && max_occurs > DEFAULT_OCCURANCE)
                    {
                        DataType::List(Box::new(d_type))
                    } else if min_occurs == max_occurs && max_occurs > DEFAULT_OCCURANCE {
                        let size = usize::try_from(max_occurs).unwrap();

                        DataType::FixedSizeList(Box::new(d_type.clone()), size)
                    } else {
                        d_type
                    };

                    let variable = Variable {
                        name: child.name.clone(),
                        xml_name: child.name.clone(),
                        requires_free: matches!(d_type, DataType::List(_) | DataType::Uri),
                        data_type: d_type,
                        required: min_occurs > 0,
                    };

                    variables.push(variable);
                }
                NodeType::Custom(c) => {
                    let c_type = registry.types.get(c);

                    if let Some(c_type) = c_type {
                        let data_type = match c_type {
                            CustomTypeDefinition::Simple(s) if s.enumeration.is_some() => {
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

                        let requires_free = match c_type {
                            CustomTypeDefinition::Simple(s) => s.list_type.is_some(),
                            CustomTypeDefinition::Complex(_) => true,
                        };

                        let data_type = if max_occurs == UNBOUNDED_OCCURANCE
                            || (min_occurs != max_occurs && max_occurs > DEFAULT_OCCURANCE)
                        {
                            DataType::List(Box::new(data_type))
                        } else if min_occurs == max_occurs && max_occurs > DEFAULT_OCCURANCE {
                            let size = usize::try_from(max_occurs).unwrap();

                            DataType::FixedSizeList(Box::new(data_type), size)
                        } else {
                            data_type
                        };

                        let variable = Variable {
                            name: child.name.clone(),
                            xml_name: child.name.clone(),
                            requires_free: requires_free
                                || matches!(
                                    data_type,
                                    DataType::List(_) | DataType::InlineList(_) | DataType::Uri
                                ),
                            data_type,
                            required: min_occurs > 0,
                        };

                        variables.push(variable);
                    }
                }
            }
        }

        let super_type = ct.base_type.as_ref().and_then(|t| {
            registry
                .types
                .get(t)
                .map(|ct| (ct.get_name(), ct.get_qualified_name()))
        });

        ClassType {
            name: ct.name.clone(),
            qualified_name: ct.qualified_name.clone(),
            super_type,
            variables,
            documentations: ct.documentations.clone(),
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
                        NodeType::Standard(s) => Self::node_base_type_to_datatype(s),
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
                }
            })
            .collect::<Vec<Variable>>()
    }

    /// Converts a list type to a data type.
    /// This is used to convert the list types of the nodes to the data types of the variables.
    ///
    /// # Arguments
    /// * `list_type` - The list type to convert.
    /// * `registry` - The type registry.
    /// # Returns
    /// The converted data type.
    fn list_type_to_data_type(list_type: &NodeType, registry: &TypeRegistry) -> Option<DataType> {
        match list_type {
            NodeType::Standard(s) => Some(Self::node_base_type_to_datatype(s)),
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

    /// Converts a node base type to a data type.
    /// This is used to convert the base types of the nodes to the data types of the variables.
    /// The base types are the types that are defined in the XML Schema specification.
    /// The data types are the types that are used in the generated code.
    ///
    /// # Arguments
    /// * `base_type` - The base type to convert.
    /// # Returns
    /// The converted data type.
    const fn node_base_type_to_datatype(base_type: &NodeBaseType) -> DataType {
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
}
