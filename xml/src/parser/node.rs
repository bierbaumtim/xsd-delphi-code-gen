use std::fs::File;
use std::io::BufReader;

use quick_xml::{events::Event, Reader};

use super::{
    annotations::AnnotationsParser,
    types::{BaseAttributes, Node, NodeType, ParserError},
};

pub struct NodeParser;

impl NodeParser {
    pub fn parse_element_with_type_node(
        reader: &mut Reader<BufReader<File>>,
        node_type: NodeType,
        name: String,
        base_attributes: BaseAttributes,
    ) -> Result<Node, ParserError> {
        let mut buf = Vec::new();
        let mut annotations = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(s)) if s.name().as_ref() == b"xs:annotation" => {
                    let mut values = AnnotationsParser::parse(reader)?;
                    annotations.append(&mut values);
                }
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"xs:element" => break,
                    _ => continue,
                },
                Ok(Event::Eof) => return Err(ParserError::UnexpectedEndOfFile),
                Err(_) => return Err(ParserError::UnexpectedError),
                _ => (),
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }

        Ok(Node::new(
            node_type,
            name,
            base_attributes,
            Some(annotations),
        ))
    }
}
