use crate::{
    generator::types::{ClassType, DataType, Variable, XMLSource},
    parser::types::{
        CustomTypeDefinition, Node, NodeType, OrderIndicator, SingleNode, DEFAULT_OCCURANCE,
        UNBOUNDED_OCCURANCE,
    },
};
use genphi_core::type_registry::TypeRegistry;

use super::helper::*;

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
pub fn build_class_type_ir(
    ct: &crate::parser::types::ComplexType,
    registry: &TypeRegistry<CustomTypeDefinition>,
) -> ClassType {
    let mut variables = collect_variables(&ct.children, registry, &ct.order);

    for attr in &ct.custom_attributes {
        match &attr.base_type {
            NodeType::Standard(s) => {
                let d_type = node_base_type_to_datatype(s);

                let variable = Variable {
                    name: attr.name.clone(),
                    xml_name: attr.name.clone(),
                    requires_free: matches!(
                        d_type,
                        DataType::List(_) | DataType::InlineList(_) | DataType::Uri
                    ),
                    data_type: d_type,
                    required: attr.required,
                    is_const: attr.fixed_value.is_some(),
                    default_value: attr.fixed_value.clone().or(attr.default_value.clone()),
                    source: XMLSource::Attribute,
                    documentations: vec![],
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

                    let variable = Variable {
                        name: attr.name.clone(),
                        xml_name: attr.name.clone(),
                        requires_free: requires_free
                            || matches!(
                                data_type,
                                DataType::List(_) | DataType::InlineList(_) | DataType::Uri
                            ),
                        data_type,
                        required: attr.required,
                        is_const: attr.fixed_value.is_some(),
                        default_value: attr.fixed_value.clone().or(attr.default_value.clone()),
                        source: XMLSource::Attribute,
                        documentations: vec![],
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

pub fn collect_variables(
    nodes: &[Node],
    registry: &TypeRegistry<CustomTypeDefinition>,
    order: &OrderIndicator,
) -> Vec<Variable> {
    nodes
        .iter()
        .filter_map(|n| match n {
            Node::Single(e) => single_node_to_variable(e, registry, order).map(|v| vec![v]),
            Node::Group(g) => Some(collect_variables(&g.nodes, registry, &g.order)),
        })
        .flatten()
        .collect::<Vec<Variable>>()
}

fn single_node_to_variable(
    node: &SingleNode,
    registry: &TypeRegistry<CustomTypeDefinition>,
    order: &OrderIndicator,
) -> Option<Variable> {
    let min_occurs = match order {
        OrderIndicator::All => node
            .base_attributes
            .min_occurs
            .unwrap_or(DEFAULT_OCCURANCE)
            .clamp(0, 1),
        OrderIndicator::Choice(base_attributes) => {
            base_attributes.min_occurs.unwrap_or(DEFAULT_OCCURANCE)
        }
        _ => node.base_attributes.min_occurs.unwrap_or(DEFAULT_OCCURANCE),
    };
    let max_occurs = match order {
        OrderIndicator::All => node
            .base_attributes
            .max_occurs
            .unwrap_or(DEFAULT_OCCURANCE)
            .clamp(0, 1),
        OrderIndicator::Choice(base_attributes) => {
            base_attributes.max_occurs.unwrap_or(DEFAULT_OCCURANCE)
        }
        _ => node.base_attributes.max_occurs.unwrap_or(DEFAULT_OCCURANCE),
    };

    let required = match order {
        OrderIndicator::Choice(_) => false,
        _ => min_occurs > 0 && !node.base_attributes.nillable,
    };

    match &node.node_type {
        NodeType::Standard(s) => {
            let d_type = node_base_type_to_datatype(s);

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

            Some(Variable {
                name: node.name.clone(),
                xml_name: node.name.clone(),
                requires_free: matches!(d_type, DataType::List(_) | DataType::Uri),
                data_type: d_type,
                required,
                default_value: None,
                is_const: false,
                source: XMLSource::Element,
                documentations: node.documentations.as_ref().cloned().unwrap_or_default(),
            })
        }
        NodeType::Custom(c) => {
            let c_type = registry.types.get(c)?;

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

            Some(Variable {
                name: node.name.clone(),
                xml_name: node.name.clone(),
                requires_free: requires_free
                    || matches!(
                        data_type,
                        DataType::List(_) | DataType::InlineList(_) | DataType::Uri
                    ),
                data_type,
                required,
                default_value: None,
                is_const: false,
                source: XMLSource::Element,
                documentations: node.documentations.as_ref().cloned().unwrap_or_default(),
            })
        }
    }
}
