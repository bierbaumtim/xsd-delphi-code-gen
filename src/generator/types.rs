use std::{fs::File, io::Write};

#[derive(Clone, Debug)]
pub(crate) enum DataType {
    Boolean,
    DateTime,
    Date,
    Double,
    Binary,
    Integer,
    String,
    Time,
    Custom(String),
}

impl DataType {
    fn get_language_representation(&self) -> String {
        match self {
            DataType::Boolean => "Boolean".to_owned(),
            DataType::DateTime => "TDateTime".to_owned(),
            DataType::Date => "TDate".to_owned(),
            DataType::Double => "Double".to_owned(),
            DataType::Binary => "TBytes".to_owned(),
            DataType::Integer => "TBytes".to_owned(),
            DataType::String => "String".to_owned(),
            DataType::Time => "TTime".to_owned(),
            DataType::Custom(c) => c.clone(),
        }
    }
}

pub(crate) trait CodeType {
    fn generate_code(&self, file: &mut File, indentation: usize) -> Result<(), std::io::Error>;
}

pub(crate) struct Enumeration {
    pub(crate) name: String,
    pub(crate) values: Vec<EnumationValue>,
}

pub(crate) struct EnumationValue {
    pub(crate) variant_name: String,
    pub(crate) xml_value: String,
}

impl CodeType for Enumeration {
    fn generate_code(&self, file: &mut File, indentation: usize) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{} = ({});\n",
            " ".repeat(indentation),
            self.name,
            self.values
                .iter()
                .map(|v| v.variant_name.clone())
                .collect::<Vec<String>>()
                .join(", ")
        ))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TypeAlias {
    pub(crate) name: String,
    pub(crate) for_type: DataType,
}

impl CodeType for TypeAlias {
    fn generate_code(&self, file: &mut File, indentation: usize) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{} = {};\n",
            " ".repeat(indentation),
            self.name,
            self.for_type.get_language_representation(),
        ))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ClassType {
    pub(crate) name: String,
    pub(crate) super_type: Option<String>,
    pub(crate) variables: Vec<Variable>,
    // local_types: Vec<ClassType>,
    // type_aliases: Vec<TypeAlias>,
    // enumerations: Vec<Enumeration>,
}

impl ClassType {
    pub(crate) fn generate_forward_declaration(
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
                .map_or_else(|| "(TObject)".to_owned(), |v| format!("(T{})", v))
        ))?;
        file.write_all(b"\n")?;
        file.write_fmt(format_args!("{}public\n", " ".repeat(indentation)))?;

        // constructors and destructors
        file.write_fmt(format_args!(
            "{}constructor FromXml(node: IXmlNode);\n",
            " ".repeat(indentation + 2),
        ))?;

        if self.variables.iter().any(|v| v.requires_free) {
            file.write_fmt(format_args!(
                "{}destructor Destroy; override;\n",
                " ".repeat(indentation + 2),
            ))?;
        }
        file.write_all(b"\n")?;
        file.write_fmt(format_args!(
            "{}function ToXmlRaw: IXmlNode;\n",
            " ".repeat(indentation + 2),
        ))?;
        // file.write_fmt(format_args!(
        //     "{}function ToXml: String;\n",
        //     " ".repeat(indentation + 2),
        // ))?;
        file.write_all(b"\n")?;

        // Variables
        for v in &self.variables {
            v.generate_code(file, indentation + 2)?;
        }

        file.write_fmt(format_args!("{}end;\n\n", " ".repeat(indentation)))?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Variable {
    pub(crate) name: String,
    pub(crate) data_type: DataType,
    pub(crate) requires_free: bool,
}

impl CodeType for Variable {
    fn generate_code(&self, file: &mut File, indentation: usize) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}{}: {};\n",
            " ".repeat(indentation),
            self.name,
            self.data_type.get_language_representation(),
        ))
    }
}
