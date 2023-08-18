use std::{borrow::Cow, fs::File, io::BufReader};

use quick_xml::{events::Event, Reader};

use super::types::ParserError;

pub(crate) struct AnnotationsParser;

impl AnnotationsParser {
    pub(crate) fn parse(reader: &mut Reader<BufReader<File>>) -> Result<Vec<String>, ParserError> {
        let mut values = Vec::new();
        let mut buf = Vec::new();
        let mut current_value = String::new();
        let mut should_read_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(s)) => match s.name().as_ref() {
                    b"xs:appinfo" | b"xs:documentation" => should_read_text = true,
                    _ => (),
                },
                Ok(Event::Text(t)) if should_read_text => {
                    let content = match t.into_inner() {
                        Cow::Borrowed(v) => {
                            String::from_utf8(v.to_vec()).map_err(|_| ParserError::UnexpectedError)
                        }
                        Cow::Owned(v) => {
                            String::from_utf8(v).map_err(|_| ParserError::UnexpectedError)
                        }
                    }?;

                    current_value.push_str(content.as_str());
                }
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"xs:appinfo" | b"xs:documentation" => {
                        should_read_text = false;

                        if !current_value.is_empty() {
                            values.push(current_value);
                            current_value = String::new();
                        }
                    }
                    b"xs:annotation" => {
                        break;
                    }
                    _ => (),
                },
                Ok(_) => (),
                Err(_) => return Err(ParserError::UnexpectedError),
            }
        }

        Ok(values)
    }
}
