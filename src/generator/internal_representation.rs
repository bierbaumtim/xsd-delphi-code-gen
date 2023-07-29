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
            (c.name.clone(), c.super_type.as_ref().map(|v| v.clone()))
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
                CustomTypeDefinition::Simple(_) => (),
                CustomTypeDefinition::Complex(ct) => {
                    let mut variables = Vec::new();

                    for child in &ct.children {
                        match &child.node_type {
                            NodeType::Standard(s) => {
                                let d_type = Self::node_base_type_to_datatype(s);

                                let variable = Variable {
                                    name: child.name.clone(),
                                    data_type: d_type,
                                    requires_free: false,
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
                                        _ => DataType::Custom(c_type.get_name()),
                                    };

                                    let variable = Variable {
                                        name: child.name.clone(),
                                        data_type,
                                        requires_free: match c_type {
                                            CustomTypeDefinition::Simple(s) => {
                                                s.list_type.is_some()
                                            }
                                            CustomTypeDefinition::Complex(_) => true,
                                        },
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
                        // local_types: Vec::new(),  // TODO
                        // type_aliases: Vec::new(), // TODO
                        // enumerations: Vec::new(), // TODO
                    };

                    classes_dep_graph.push(class_type);
                }
            }
        }

        let mut document_variables = Vec::new();

        for node in nodes {
            let variable = Variable {
                name: node.name.clone(),
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
            NodeBaseType::Integer => DataType::Integer,
            NodeBaseType::String => DataType::String,
            NodeBaseType::Time => DataType::Time,
        }
    }
}
