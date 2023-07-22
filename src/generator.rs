use std::{fs::File, io::Write};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    parser_types::{CustomTypeDefinition, Node, NodeBaseType, NodeType, SimpleType},
    type_registry::TypeRegistry,
};

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
        self.build_ir(registry);

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
        self.file.write_all(b"uses System.Types;")?;
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

        self.file.write_all(b"  // Enumerations\n")?;
        for e in &self.enumerations {
            e.generate_code(self.file, 2)?;
        }

        self.newline()?;

        self.file.write_all(b"  // Aliases\n")?;
        for a in &self.types_aliases {
            a.generate_code(self.file, 2)?;
        }

        self.newline()?;

        self.file.write_all(b"  // Forward Declarations\n")?;
        for c in &self.classes {
            c.generate_forward_declaration(self.file, 2)?;
        }

        self.newline()?;

        Ok(())
    }

    fn write_declarations(&mut self, registry: &TypeRegistry) -> Result<(), std::io::Error> {
        self.file.write_all(b"  // Declarations\n")?;
        for c in &self.classes {
            c.generate_code(self.file, 2)?;
        }

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

    fn build_ir(&mut self, registry: &TypeRegistry) {
        for c_type in registry.types.values() {
            match c_type {
                CustomTypeDefinition::Simple(st) if st.enumeration.is_some() => {
                    let enumeration = self.build_enumeration_ir(st);

                    self.enumerations.push(enumeration);
                }
                CustomTypeDefinition::Simple(st) if !st.is_local && st.base_type.is_some() => {
                    let alias = self.build_type_alias_ir(st);

                    self.types_aliases.push(alias);
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
                                    type_name: d_type,
                                };

                                variables.push(variable);
                            }
                            NodeType::Custom(c) => {
                                let c_type = registry.types.get(c);

                                if let Some(c_type) = c_type {
                                    let variable = Variable {
                                        name: self.first_char_uppercase(&child.name),
                                        type_name: self.as_type_name(&c_type.get_name()),
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
                            .map(|ct| self.as_type_name(&ct.get_name())),
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

                    self.classes.push(class_type);
                }
            }
        }

        // TODO: Write document type
    }

    fn build_enumeration_ir(&self, st: &SimpleType) -> Enumeration {
        let capitalized_name = self.first_char_uppercase(&st.name);

        let values = st
            .enumeration
            .as_ref()
            .unwrap()
            .iter()
            .map(|v| self.first_char_lowercase(v))
            .collect::<Vec<String>>();

        Enumeration {
            name: capitalized_name,
            values,
        }
    }

    fn build_type_alias_ir(&self, st: &SimpleType) -> TypeAlias {
        let capitalized_name = self.first_char_uppercase(&st.name);
        let for_type = match st.base_type.as_ref().unwrap() {
            NodeType::Standard(t) => self.node_base_type_to_datatype(t),
            NodeType::Custom(n) => self.as_type_name(&n),
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

    fn node_base_type_to_datatype(&self, base_type: &NodeBaseType) -> String {
        match base_type {
            NodeBaseType::Boolean => "Boolean",
            NodeBaseType::DateTime => "TDateTime",
            NodeBaseType::Date => "TDate",
            NodeBaseType::Decimal | NodeBaseType::Double | NodeBaseType::Float => "Double",
            NodeBaseType::HexBinary | NodeBaseType::Base64Binary => "TBytes",
            NodeBaseType::Integer => "Integer",
            NodeBaseType::String => "String",
            NodeBaseType::Time => "TTime",
        }
        .to_owned()
    }
}

trait CodeType {
    fn generate_code(&self, file: &mut File, indentation: usize) -> Result<(), std::io::Error>;
}

struct Enumeration {
    name: String,
    values: Vec<String>,
}

impl CodeType for Enumeration {
    fn generate_code(&self, file: &mut File, indentation: usize) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{} = ({});\n",
            " ".repeat(indentation),
            self.name,
            self.values.join(", ")
        ))
    }
}

struct TypeAlias {
    name: String,
    for_type: String,
}

impl CodeType for TypeAlias {
    fn generate_code(&self, file: &mut File, indentation: usize) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{} = {};\n",
            " ".repeat(indentation),
            self.name,
            self.for_type
        ))
    }
}

struct ClassType {
    name: String,
    super_type: Option<String>,
    variables: Vec<Variable>,
    // local_types: Vec<ClassType>,
    // type_aliases: Vec<TypeAlias>,
    // enumerations: Vec<Enumeration>,
}

impl ClassType {
    fn generate_forward_declaration(
        &self,
        file: &mut File,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{} = class;\n",
            " ".repeat(indentation),
            self.name,
        ))
    }
}

impl CodeType for ClassType {
    fn generate_code(&self, file: &mut File, indentation: usize) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{} = class{}",
            " ".repeat(indentation),
            self.name,
            self.super_type
                .as_ref()
                .map_or_else(|| "(TObject)".to_owned(), |v| format!("({})", v))
        ))?;
        file.write_all(b"\n")?;
        file.write_fmt(format_args!("{}public\n", " ".repeat(indentation)))?;

        // Variables
        for v in &self.variables {
            v.generate_code(file, indentation + 2)?;
        }

        file.write_fmt(format_args!("{}end;\n\n", " ".repeat(indentation)))?;

        Ok(())
    }
}

struct Variable {
    name: String,
    type_name: String,
}

impl CodeType for Variable {
    fn generate_code(&self, file: &mut File, indentation: usize) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}{}: {};\n",
            " ".repeat(indentation),
            self.name,
            self.type_name
        ))
    }
}
