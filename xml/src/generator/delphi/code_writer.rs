use std::io::{BufWriter, Write};

/// A helper struct to write code to a buffer.
pub struct CodeWriter<T: Write> {
    pub(crate) buffer: BufWriter<T>,
}
