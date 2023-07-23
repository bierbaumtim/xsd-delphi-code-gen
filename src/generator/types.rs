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

pub(crate) trait DeclarationType {
    fn generate_code(&self, file: &mut File, indentation: usize) -> Result<(), std::io::Error>;
}

pub(crate) struct Enumeration {
    pub(crate) name: String,
    pub(crate) values: Vec<EnumerationValue>,
}

pub(crate) struct EnumerationValue {
    pub(crate) variant_name: String,
    pub(crate) xml_value: String,
}

impl Enumeration {
    pub(crate) fn generate_declarations_code(
        &self,
        file: &mut File,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
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

    pub(crate) fn generate_helper_declaration_code(
        &self,
        file: &mut File,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{}Helper = record helper for T{}\n",
            " ".repeat(indentation),
            self.name,
            self.name,
        ))?;
        file.write_fmt(format_args!(
            "{}class function FromXmlValue(const pXmlValue: String): T{}; static;\n",
            " ".repeat(indentation + 2),
            self.name,
        ))?;
        file.write_fmt(format_args!(
            "{}function ToXmlValue: String;\n",
            " ".repeat(indentation + 2),
        ))?;
        file.write_fmt(format_args!("{}end;", " ".repeat(indentation),))?;
        file.write(b"\n")?;
        file.write(b"\n")?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TypeAlias {
    pub(crate) name: String,
    pub(crate) for_type: DataType,
}

impl DeclarationType for TypeAlias {
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

impl DeclarationType for ClassType {
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
            "{}constructor FromXml(node: IXMLNode);\n",
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
            "{}function ToXmlRaw: IXMLNode;\n",
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

impl DeclarationType for Variable {
    fn generate_code(&self, file: &mut File, indentation: usize) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}{}: {};\n",
            " ".repeat(indentation),
            self.name,
            self.data_type.get_language_representation(),
        ))
    }
}
