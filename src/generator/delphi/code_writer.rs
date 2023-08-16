use std::io::{BufWriter, Write};

pub(crate) struct CodeWriter<'a, T: Write> {
    buffer: &'a mut BufWriter<T>,
}

impl<'a, T: Write> CodeWriter<'a, T> {
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
            "{}{}\n",
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
}
