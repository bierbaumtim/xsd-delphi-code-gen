use crate::{parser::types::*, type_registry::TypeRegistry};

use super::{dependency_graph::DependencyGraph, types::*};

pub(crate) const DOCUMENT_NAME: &str = "Document";

pub(crate) struct InternalRepresentation {
    pub(crate) document: ClassType,
    pub(crate) classes: Vec<ClassType>,
    pub(crate) types_aliases: Vec<TypeAlias>,
    pub(crate) enumerations: Vec<Enumeration>,
    pub(crate) union_types: Vec<UnionType>,
}

impl InternalRepresentation {
    pub(crate) fn build(data: &ParsedData, registry: &TypeRegistry) -> InternalRepresentation {
        let mut classes_dep_graph = DependencyGraph::<String, ClassType, _>::new(|c| {
            (
                c.name.clone(),
                c.super_type.as_ref().cloned().map(|s| vec![s]),
            )
        });
        let mut aliases_dep_graph =
            DependencyGraph::<String, TypeAlias, _>::new(|a| match &a.for_type {
                DataType::Custom(name) => (a.name.clone(), Some(vec![name.clone()])),
                _ => (a.name.clone(), None),
            });
        let mut union_types_dep_graph = DependencyGraph::<String, UnionType, _>::new(|u| {
            (
                u.name.clone(),
                Some(
                    u.variants
                        .iter()
                        .map(|v| match &v.data_type {
                            DataType::Union(n) => n.clone(),
                            _ => String::new(),
                        })
                        .filter(|d| !d.is_empty())
                        .collect::<Vec<String>>(),
                ),
            )
        });

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
                                } else if min_occurs == max_occurs && max_occurs > DEFAULT_OCCURANCE
                                {
                                    let size = usize::try_from(max_occurs).unwrap();

                                    DataType::FixedSizeList(Box::new(d_type.clone()), size)
                                } else {
                                    d_type
                                };

                                let variable = Variable {
                                    name: child.name.clone(),
                                    xml_name: child.name.clone(),
                                    requires_free: matches!(d_type, DataType::List(_)),
                                    data_type: d_type,
                                };

                                variables.push(variable);
                            }
                            NodeType::Custom(c) => {
                                let c_type = registry.types.get(c);

                                if let Some(c_type) = c_type {
                                    let data_type = match c_type {
                                        CustomTypeDefinition::Simple(s)
                                            if s.enumeration.is_some() =>
                                        {
                                            DataType::Enumeration(s.name.clone())
                                        }
                                        CustomTypeDefinition::Simple(s)
                                            if s.base_type.is_some() =>
                                        {
                                            DataType::Alias(s.name.clone())
                                        }
                                        CustomTypeDefinition::Simple(s)
                                            if s.list_type.is_some() =>
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
                                    };

                                    let variable = Variable {
                                        name: child.name.clone(),
                                        xml_name: child.name.clone(),
                                        requires_free: requires_free
                                            || matches!(
                                                data_type,
                                                DataType::List(_) | DataType::InlineList(_)
                                            ),
                                        data_type,
                                    };

                                    variables.push(variable);
                                }
                            }
                        }
                    }

                    let super_type = match &ct.base_type {
                        Some(t) => registry.types.get(t).map(|ct| ct.get_name()),
                        None => None,
                    };

                    let class_type = ClassType {
                        name: ct.name.clone(),
                        qualified_name: ct.qualified_name.clone(),
                        super_type,
                        variables,
                        documentations: ct.documentations.clone(),
                    };

                    classes_dep_graph.push(class_type);
                }
            }
        }

        let mut document_variables = Vec::new();

        for node in &data.nodes {
            let variable = Variable {
                name: node.name.clone(),
                xml_name: node.name.clone(),
                data_type: match &node.node_type {
                    NodeType::Standard(s) => Self::node_base_type_to_datatype(s),
                    NodeType::Custom(e) => {
                        let c_type = registry.types.get(e);

                        match c_type {
                            Some(c) => DataType::Custom(c.get_name()),
                            None => todo!(),
                        }
                    }
                },
                requires_free: match &node.node_type {
                    NodeType::Standard(_) => false,
                    NodeType::Custom(c) => {
                        let c_type = registry.types.get(c);

                        match c_type {
                            Some(t) => match t {
                                CustomTypeDefinition::Simple(s) => s.list_type.is_some(),
                                CustomTypeDefinition::Complex(_) => true,
                            },
                            None => false,
                        }
                    }
                },
            };

            document_variables.push(variable);
        }

        let document_type = ClassType {
            super_type: None,
            name: String::from(DOCUMENT_NAME),
            qualified_name: String::from(DOCUMENT_NAME),
            variables: document_variables,
            documentations: vec![],
        };

        classes_dep_graph.push(document_type.clone());

        InternalRepresentation {
            document: document_type,
            classes: classes_dep_graph.get_sorted_elements(),
            types_aliases: aliases_dep_graph.get_sorted_elements(),
            union_types: union_types_dep_graph.get_sorted_elements(),
            enumerations,
        }
    }

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

    fn build_union_type_ir(st: &SimpleType, registry: &TypeRegistry) -> UnionType {
        UnionType {
            name: st.name.clone(),
            qualified_name: st.qualified_name.clone(),
            documentations: st.documentations.clone(),
            variants: st.variants.as_ref().unwrap()
                .iter()
                .enumerate()
                .filter_map(|(i, v)| {
                    let d_type = match v {
                        crate::parser::types::UnionVariant::Named(n) => {
                            let Some(CustomTypeDefinition::Simple(st)) = registry.types.get(n) else {
                                return None;
                            };

                            if let Some(lt) = &st.list_type {
                                Self::list_type_to_data_type(lt, registry)
                                    .map(|d| (d, st.name.clone()))
                            } else if st.enumeration.is_some() {
                                Some((
                                    DataType::Enumeration(st.name.clone()),
                                    st.name.clone(),
                                ))
                            } else {
                                Some((
                                    DataType::Alias(st.name.clone()),
                                    st.name.clone(),
                                ))
                            }
                        }
                        crate::parser::types::UnionVariant::Simple(st) => {
                            if let Some(lt) = &st.list_type {
                                Self::list_type_to_data_type(lt, registry)
                                    .map(|d| (d, st.name.clone()))
                            } else if st.enumeration.is_some() {
                                Some((
                                    DataType::Enumeration(st.name.clone()),
                                    st.name.clone(),
                                ))
                            } else {
                                Some((
                                    DataType::Alias(st.name.clone()),
                                    st.name.clone(),
                                ))
                            }
                        }
                        crate::parser::types::UnionVariant::Standard(t) => Some((
                            Self::node_base_type_to_datatype(t),
                            format!("Variant{}", i),
                        )),
                    };

                    d_type.map(|(dt, name)| super::types::UnionVariant {
                        name,
                        data_type: dt,
                    })
                })
                .collect::<Vec<super::types::UnionVariant>>(),
        }
    }

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

    fn node_base_type_to_datatype(base_type: &NodeBaseType) -> DataType {
        match base_type {
            NodeBaseType::Boolean => DataType::Boolean,
            NodeBaseType::DateTime => DataType::DateTime,
            NodeBaseType::Date => DataType::Date,
            NodeBaseType::Decimal | NodeBaseType::Double | NodeBaseType::Float => DataType::Double,
            NodeBaseType::HexBinary => DataType::Binary(BinaryEncoding::Hex),
            NodeBaseType::Base64Binary => DataType::Binary(BinaryEncoding::Base64),
            NodeBaseType::String => DataType::String,
            NodeBaseType::Time => DataType::Time,
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
