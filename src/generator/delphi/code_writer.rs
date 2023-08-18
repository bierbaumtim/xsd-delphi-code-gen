use std::io::{BufWriter, Write};

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
}

#[cfg(test)]
impl<T: Write> CodeWriter<T> {
    pub(crate) fn get_writer(&mut self) -> Result<&T, std::io::Error> {
        self.buffer.flush()?;

        Ok(self.buffer.get_ref())
    }
}
