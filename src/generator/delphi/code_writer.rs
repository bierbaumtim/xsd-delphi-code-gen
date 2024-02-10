use std::io::{BufWriter, Write};

pub type FunctionParameter<'a> = (&'a str, &'a str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionType {
    Procedure,
    Function(String),
}

impl FunctionType {
    const fn as_text(&self) -> &str {
        match self {
            Self::Procedure => "procedure",
            Self::Function(_) => "function",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionModifier {
    Virtual,
    Override,
}

impl FunctionModifier {
    const fn as_text(&self) -> &str {
        match self {
            Self::Virtual => "virtual",
            Self::Override => "override",
        }
    }
}

/// A helper struct to write code to a buffer.
pub struct CodeWriter<T: Write> {
    pub(crate) buffer: BufWriter<T>,
}

impl<T: Write> CodeWriter<T> {
    /// Write a newline to the buffer.
    #[inline]
    pub(crate) fn newline(&mut self) -> Result<(), std::io::Error> {
        self.buffer.write_all(b"\n")
    }

    /// Write a string to the buffer, and optionally indent it.
    #[inline]
    pub(crate) fn write(
        &mut self,
        content: &str,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        self.buffer.write_fmt(format_args!(
            "{}{}",
            " ".repeat(indentation.unwrap_or(0)),
            content
        ))
    }

    /// Write a string to the buffer, and optionally indent it followed by a newline.
    pub(crate) fn writeln(
        &mut self,
        content: &str,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        self.write(content, indentation)?;
        self.newline()
    }

    /// Write a string to the buffer, and optionally indent it followed by a newline.
    pub(crate) fn writeln_fmt(
        &mut self,
        content: std::fmt::Arguments<'_>,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        if let Some(indentation) = indentation {
            self.buffer.write_all(" ".repeat(indentation).as_bytes())?;
        }
        self.buffer.write_fmt(content)?;
        self.newline()
    }

    /// Write a string in a documentation comment to the buffer, and optionally indent it.
    pub(crate) fn write_documentation(
        &mut self,
        documentations: &Vec<String>,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        for documentation in documentations {
            for line in documentation.split('\n') {
                self.writeln_fmt(format_args!("// {line}"), indentation)?;
            }
        }

        Ok(())
    }

    /// Write default constructor delcaration to the buffer, and optionally indent it.
    pub(crate) fn write_default_constructor(
        &mut self,
        modifier: Option<FunctionModifier>,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        if let Some(indentation) = indentation {
            self.buffer.write_all(" ".repeat(indentation).as_bytes())?;
        }

        self.buffer.write_all(b"constructor Create;")?;

        if let Some(modifier) = modifier {
            self.buffer
                .write_fmt(format_args!(" {};", modifier.as_text()))?;
        }

        self.newline()?;

        Ok(())
    }

    /// Write constructor with parameters declaration to the buffer, and optionally indent it.
    pub fn write_constructor(
        &mut self,
        name: &str,
        parameters: Option<Vec<FunctionParameter>>,
        modifier: Option<FunctionModifier>,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        if let Some(indentation) = indentation {
            self.buffer.write_all(" ".repeat(indentation).as_bytes())?;
        }

        self.buffer.write_fmt(format_args!("constructor {name}"))?;

        if let Some(parameters) = parameters {
            self.buffer.write_all(b"(")?;

            for (i, param) in parameters.iter().enumerate() {
                self.buffer
                    .write_fmt(format_args!("{}: {}", param.0, param.1))?;

                if i < parameters.len() - 1 {
                    self.buffer.write_all(b"; ")?;
                }
            }

            self.buffer.write_all(b")")?;
        }

        self.buffer.write_all(b";")?;

        if let Some(modifier) = modifier {
            self.buffer
                .write_fmt(format_args!(" {};", modifier.as_text()))?;
        }

        self.newline()?;

        Ok(())
    }

    /// Write destructor declaration to the buffer, and optionally indent it.
    pub(crate) fn write_destructor(
        &mut self,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        self.writeln("destructor Destroy; override;", indentation)
    }

    /// Write function declaration to the buffer, and optionally indent it.
    ///
    /// # Arguments
    ///
    /// * `f_type` - The type of the function.
    /// * `name` - The name of the function.
    /// * `parameters` - The parameters of the function.
    /// * `is_class_function` - Whether the function is a class function.
    /// * `modifiers` - The modifiers of the function.
    /// * `indentation` - The indentation of the function.
    pub(crate) fn write_function_declaration(
        &mut self,
        f_type: FunctionType,
        name: &str,
        parameters: Option<Vec<FunctionParameter>>,
        is_class_function: bool,
        modifiers: Option<Vec<FunctionModifier>>,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        if indentation > 0 {
            self.buffer.write_all(" ".repeat(indentation).as_bytes())?;
        }

        if is_class_function {
            self.buffer.write_all(b"class ")?;
        }

        self.buffer
            .write_fmt(format_args!("{} {}", f_type.as_text(), name))?;

        if let Some(parameters) = parameters {
            self.buffer.write_all(b"(")?;

            for (i, param) in parameters.iter().enumerate() {
                self.buffer
                    .write_fmt(format_args!("{}: {}", param.0, param.1))?;

                if i < parameters.len() - 1 {
                    self.buffer.write_all(b"; ")?;
                }
            }

            self.buffer.write_all(b")")?;
        }

        if let FunctionType::Function(return_type) = f_type {
            self.buffer.write_fmt(format_args!(": {return_type};"))?;
        } else {
            self.buffer.write_all(b";")?;
        }

        if is_class_function {
            self.buffer.write_all(b" static;")?;
        }

        if let Some(modifiers) = modifiers {
            for modifier in &modifiers {
                self.buffer
                    .write_fmt(format_args!(" {};", modifier.as_text()))?;
            }
        }

        self.newline()?;

        Ok(())
    }

    /// Write a variable initialization to the buffer, and optionally indent it.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable.
    /// * `type_name` - The type of the variable.
    /// * `is_required` - Whether the variable is required.
    /// * `is_value_type` - Whether the variable is a value type.
    /// * `indentation` - The indentation of the variable.
    pub(crate) fn write_variable_initialization(
        &mut self,
        name: &str,
        type_name: &str,
        is_required: bool,
        is_value_type: bool,
        default_value: Option<String>,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        match (is_required, is_value_type, default_value) {
            (false, false, _) => self.writeln_fmt(format_args!("{name} := nil;"), indentation)?,
            (false, true, None) => self.writeln_fmt(
                format_args!("{name} := TNone<{type_name}>.Create;"),
                indentation,
            )?,
            (true, false, _) => {
                self.writeln_fmt(format_args!("{name} := {type_name}.Create;"), indentation)?;
            }
            (true, true, None) => {
                self.writeln_fmt(format_args!("{name} := Default({type_name});"), indentation)?;
            }
            (_, true, Some(v)) => self.writeln_fmt(format_args!("{name} := {v};"), indentation)?,
        }

        Ok(())
    }
}

#[cfg(test)]
impl<T: Write> CodeWriter<T> {
    pub(crate) fn get_writer(&mut self) -> Result<&T, std::io::Error> {
        self.buffer.flush()?;

        Ok(self.buffer.get_ref())
    }
}
