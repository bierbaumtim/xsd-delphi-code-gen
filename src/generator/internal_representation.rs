use crate::{parser_types::*, type_registry::TypeRegistry};

use super::{dependency_graph::DependencyGraph, types::*};

pub(crate) const DOCUMENT_NAME: &str = "Document";

pub(crate) struct InternalRepresentation {
    pub(crate) document: ClassType,
    pub(crate) classes: Vec<ClassType>,
    pub(crate) types_aliases: Vec<TypeAlias>,
    pub(crate) enumerations: Vec<Enumeration>,
}

impl InternalRepresentation {
    pub(crate) fn build(nodes: &Vec<Node>, registry: &TypeRegistry) -> InternalRepresentation {
        let mut classes_dep_graph = DependencyGraph::<String, ClassType, _>::new(|c| {
            (c.name.clone(), c.super_type.as_ref().cloned())
        });
        let mut aliases_dep_graph =
            DependencyGraph::<String, TypeAlias, _>::new(|a| match &a.for_type {
                DataType::Custom(name) => (a.name.clone(), Some(name.clone())),
                _ => (a.name.clone(), None),
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
                        match lt {
                            NodeType::Standard(s) => {
                                let d_type = Self::node_base_type_to_datatype(s);

                                let type_alias = TypeAlias {
                                    name: st.name.clone(),
                                    for_type: DataType::List(Box::new(d_type)),
                                    pattern: None,
                                };

                                aliases_dep_graph.push(type_alias);
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
                                        _ => DataType::Custom(c_type.get_name()),
                                    };

                                    let type_alias = TypeAlias {
                                        name: st.name.clone(),
                                        for_type: DataType::List(Box::new(data_type)),
                                        pattern: None,
                                    };

                                    aliases_dep_graph.push(type_alias);
                                }
                            }
                        }
                    }
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

                                let d_type = if max_occurs == UNBOUNDED_OCCURANCE {
                                    DataType::List(Box::new(d_type))
                                } else if min_occurs != max_occurs && max_occurs > DEFAULT_OCCURANCE
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
                                            panic!("Nested lists are not supported");
                                            // return Err();
                                        }
                                        _ => DataType::Custom(c_type.get_name()),
                                    };

                                    let requires_free = match c_type {
                                        CustomTypeDefinition::Simple(s) => s.list_type.is_some(),
                                        CustomTypeDefinition::Complex(_) => true,
                                    };

                                    let data_type = if max_occurs == UNBOUNDED_OCCURANCE {
                                        DataType::List(Box::new(data_type))
                                    } else if min_occurs != max_occurs
                                        && max_occurs > DEFAULT_OCCURANCE
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
                                            || matches!(data_type, DataType::List(_)),
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
                        super_type,
                        variables,
                    };

                    classes_dep_graph.push(class_type);
                }
            }
        }

        let mut document_variables = Vec::new();

        for node in nodes {
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
            variables: document_variables,
        };

        classes_dep_graph.push(document_type.clone());

        InternalRepresentation {
            document: document_type,
            classes: classes_dep_graph.get_sorted_elements(),
            types_aliases: aliases_dep_graph.get_sorted_elements(),
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
                variant_name: v.clone(),
                xml_value: v.clone(),
            })
            .collect::<Vec<EnumerationValue>>();

        Enumeration {
            name: st.name.clone(),
            values,
        }
    }

    fn build_type_alias_ir(st: &SimpleType) -> TypeAlias {
        let for_type = match st.base_type.as_ref().unwrap() {
            NodeType::Standard(t) => Self::node_base_type_to_datatype(t),
            NodeType::Custom(n) => DataType::Custom(n.clone()),
        };

        TypeAlias {
            name: st.name.clone(),
            pattern: st.pattern.clone(),
            for_type,
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
