use std::io::{BufWriter, Write};

pub(crate) type FunctionParameter<'a> = (&'a str, &'a str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FunctionType {
    Procedure,
    Function(String),
}

impl FunctionType {
    fn as_text(&self) -> &str {
        match self {
            FunctionType::Procedure => "procedure",
            FunctionType::Function(_) => "function",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FunctionModifier {
    Virtual,
    Override,
}

impl FunctionModifier {
    fn as_text(&self) -> &str {
        match self {
            FunctionModifier::Virtual => "virtual",
            FunctionModifier::Override => "override",
        }
    }
}

pub(crate) struct CodeWriter<T: Write> {
    pub(crate) buffer: BufWriter<T>,
}

impl<T: Write> CodeWriter<T> {
    #[inline]
    pub(crate) fn newline(&mut self) -> Result<(), std::io::Error> {
        self.buffer.write_all(b"\n")
    }

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

    pub(crate) fn writeln(
        &mut self,
        content: &str,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        self.write(content, indentation)?;
        self.newline()
    }

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

    pub(crate) fn write_documentation(
        &mut self,
        documentations: &Vec<String>,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        for documentation in documentations {
            for line in documentation.split('\n') {
                self.writeln_fmt(format_args!("// {}", line), indentation)?;
            }
        }

        Ok(())
    }

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

        self.buffer
            .write_fmt(format_args!("constructor {}", name))?;

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

    pub(crate) fn write_destructor(
        &mut self,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        self.writeln("destructor Destroy; override;", indentation)
    }

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
            self.buffer.write_fmt(format_args!(": {};", return_type))?;
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

    pub(crate) fn write_variable_initialization(
        &mut self,
        name: &str,
        type_name: &str,
        is_required: bool,
        is_value_type: bool,
        indentation: Option<usize>,
    ) -> Result<(), std::io::Error> {
        match (is_required, is_value_type) {
            (false, false) => self.writeln_fmt(format_args!("{} := nil;", name), indentation)?,
            (false, true) => self.writeln_fmt(
                format_args!("{} := TNone<{}>.Create;", name, type_name),
                indentation,
            )?,
            (true, false) => self.writeln_fmt(
                format_args!("{} := {}.Create;", name, type_name),
                indentation,
            )?,
            (true, true) => self.writeln_fmt(
                format_args!("{} := Default({});", name, type_name),
                indentation,
            )?,
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
