use std::{fs::File, io::Write};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    generator::{dependency_graph::DependencyGraph, types::*},
    parser_types::{CustomTypeDefinition, Node, NodeBaseType, NodeType, SimpleType},
    type_registry::TypeRegistry,
};

// TODO: No forward declaration for document
// TODO: build IR(Intermediate Representation) with more informations about DataType, Inheritance
// TODO: Sort Document class first
// TODO: Sort class Declarations by occurance in document, then by inheritance and dependency

pub(crate) struct Generator<'a> {
    file: &'a mut File,
    unit_name: String,
    enumerations: Vec<Enumeration>,
    types_aliases: Vec<TypeAlias>,
    classes: Vec<ClassType>,
}

impl<'a> Generator<'a> {
    pub(crate) fn new(file: &'a mut File, unit_name: String) -> Generator {
        Generator {
            file,
            unit_name: unit_name.clone(),
            enumerations: Vec::new(),
            classes: Vec::new(),
            types_aliases: Vec::new(),
        }
    }

    pub(crate) fn generate(
        &mut self,
        nodes: Vec<Node>,
        registry: &TypeRegistry,
    ) -> Result<(), std::io::Error> {
        self.build_ir(&nodes, registry);

        self.write_unit()?;
        self.write_interface_start()?;
        self.write_uses()?;

        self.write_forward_declerations(registry)?;
        self.write_declarations(registry)?;

        self.write_implementation_start()?;
        // TODO: write implementation

        self.write_file_end()?;
        Ok(())
    }

    fn write_unit(&mut self) -> Result<(), std::io::Error> {
        self.file
            .write_fmt(format_args!("unit {};", self.unit_name))?;
        self.newline()?;
        self.newline()
    }

    fn write_uses(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"uses System.Types,\n")?;
        self.file.write_all(b"     System.Xml;")?;
        self.newline()?;
        self.newline()
    }

    fn write_interface_start(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"interface")?;
        self.newline()?;
        self.newline()
    }

    fn write_forward_declerations(
        &mut self,
        registry: &TypeRegistry,
    ) -> Result<(), std::io::Error> {
        self.file.write(b"types")?;
        self.newline()?;
        self.newline()?;

        if !self.enumerations.is_empty() {
            self.file.write_all(b"  {$REGION 'Enumerations'}\n")?;
            for e in &self.enumerations {
                e.generate_declarations_code(self.file, 2)?;
            }
            self.file.write_all(b"  {$ENDREGION}\n")?;

            self.newline()?;
            self.file
                .write_all(b"  {$REGION 'Enumerations Helper'}\n")?;
            for e in &self.enumerations {
                e.generate_helper_declaration_code(self.file, 2)?;
            }
            self.file.write_all(b"  {$ENDREGION}\n")?;
            self.newline()?;
        }

        if !self.types_aliases.is_empty() {
            self.file.write_all(b"  {$REGION 'Aliases'}\n")?;
            for a in &self.types_aliases {
                a.generate_code(self.file, 2)?;
            }
            self.file.write_all(b"  {$ENDREGION}\n")?;
            self.newline()?;
        }

        if !self.classes.is_empty() {
            self.file
                .write_all(b"  {$REGION 'Forward Declarations}\n")?;
            for c in &self.classes {
                c.generate_forward_declaration(self.file, 2)?;
            }
            self.file.write_all(b"  {$ENDREGION}\n")?;
            self.newline()?;
        }

        Ok(())
    }

    fn write_declarations(&mut self, registry: &TypeRegistry) -> Result<(), std::io::Error> {
        self.file.write_all(b"  {$REGION 'Declarations}\n")?;
        for c in &self.classes {
            c.generate_code(self.file, 2)?;
        }
        self.file.write_all(b"  {$ENDREGION}\n")?;

        self.newline()?;

        Ok(())
    }

    fn write_implementation_start(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"implementation")?;
        self.newline()?;
        self.newline()
    }

    fn write_file_end(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"end.")
    }

    fn newline(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"\n")
    }

    fn build_ir(&mut self, nodes: &Vec<Node>, registry: &TypeRegistry) {
        let mut classes_dep_graph = DependencyGraph::<String, ClassType, _>::new(|c| {
            (c.name.clone(), c.super_type.as_ref().map(|v| v.clone()))
        });
        let mut aliases_dep_graph =
            DependencyGraph::<String, TypeAlias, _>::new(|a| match &a.for_type {
                DataType::Custom(name) => (a.name.clone(), Some(name.clone())),
                _ => (a.name.clone(), None),
            });

        for c_type in registry.types.values() {
            match c_type {
                CustomTypeDefinition::Simple(st) if st.enumeration.is_some() => {
                    let enumeration = self.build_enumeration_ir(st);

                    self.enumerations.push(enumeration);
                }
                CustomTypeDefinition::Simple(st) if !st.is_local && st.base_type.is_some() => {
                    let alias = self.build_type_alias_ir(st);

                    aliases_dep_graph.push(alias);
                }
                CustomTypeDefinition::Simple(_) => (),
                CustomTypeDefinition::Complex(ct) => {
                    let capitalized_name = self.first_char_uppercase(&ct.name);
                    let mut variables = Vec::new();

                    for child in &ct.children {
                        match &child.node_type {
                            NodeType::Standard(s) => {
                                let d_type = self.node_base_type_to_datatype(s);

                                let variable = Variable {
                                    name: self.first_char_uppercase(&child.name),
                                    data_type: d_type,
                                    requires_free: false,
                                };

                                variables.push(variable);
                            }
                            NodeType::Custom(c) => {
                                let c_type = registry.types.get(c);

                                if let Some(c_type) = c_type {
                                    let variable = Variable {
                                        name: self.first_char_uppercase(&child.name),
                                        data_type: DataType::Custom(
                                            self.as_type_name(&c_type.get_name()),
                                        ),
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
                        Some(t) => registry
                            .types
                            .get(t)
                            .map(|ct| self.first_char_uppercase(&ct.get_name())),
                        None => None,
                    };

                    let class_type = ClassType {
                        name: capitalized_name,
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
                    NodeType::Standard(s) => self.node_base_type_to_datatype(s),
                    NodeType::Custom(e) => {
                        let c_type = registry.types.get(e);

                        match c_type {
                            Some(c) => DataType::Custom(self.as_type_name(&c.get_name())),
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
            name: "Document".to_owned(),
            variables: document_variables,
        };

        classes_dep_graph.push(document_type);

        self.classes = classes_dep_graph.get_sorted_elements();
        self.types_aliases = aliases_dep_graph.get_sorted_elements();
        self.enumerations.sort_by_key(|e| e.name.clone());
    }

    fn build_enumeration_ir(&self, st: &SimpleType) -> Enumeration {
        let capitalized_name = self.first_char_uppercase(&st.name);

        let values = st
            .enumeration
            .as_ref()
            .unwrap()
            .iter()
            .map(|v| EnumerationValue {
                variant_name: self.first_char_lowercase(v),
                xml_value: v.clone(),
            })
            .collect::<Vec<EnumerationValue>>();

        Enumeration {
            name: capitalized_name,
            values,
        }
    }

    fn build_type_alias_ir(&self, st: &SimpleType) -> TypeAlias {
        let capitalized_name = self.first_char_uppercase(&st.name);
        let for_type = match st.base_type.as_ref().unwrap() {
            NodeType::Standard(t) => self.node_base_type_to_datatype(t),
            NodeType::Custom(n) => DataType::Custom(self.as_type_name(&n)),
        };

        TypeAlias {
            name: capitalized_name,
            for_type,
        }
    }

    fn first_char_uppercase(&self, name: &String) -> String {
        let mut graphemes = name.graphemes(true);

        match graphemes.next() {
            None => String::new(),
            Some(c) => c.to_uppercase() + graphemes.as_str(),
        }
    }

    fn first_char_lowercase(&self, name: &String) -> String {
        let mut graphemes = name.graphemes(true);

        match graphemes.next() {
            None => String::new(),
            Some(c) => c.to_lowercase() + graphemes.as_str(),
        }
    }

    fn as_type_name(&self, name: &String) -> String {
        String::from("T") + self.first_char_uppercase(name).as_str()
    }

    fn node_base_type_to_datatype(&self, base_type: &NodeBaseType) -> DataType {
        match base_type {
            NodeBaseType::Boolean => DataType::Boolean,
            NodeBaseType::DateTime => DataType::DateTime,
            NodeBaseType::Date => DataType::Date,
            NodeBaseType::Decimal | NodeBaseType::Double | NodeBaseType::Float => DataType::Double,
            NodeBaseType::HexBinary | NodeBaseType::Base64Binary => DataType::Binary,
            NodeBaseType::Integer => DataType::Integer,
            NodeBaseType::String => DataType::String,
            NodeBaseType::Time => DataType::Time,
        }
    }
}
