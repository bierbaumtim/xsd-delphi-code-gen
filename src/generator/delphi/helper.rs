use unicode_segmentation::UnicodeSegmentation;

use crate::generator::types::DataType;

pub(crate) struct Helper;

impl Helper {
    pub(crate) fn first_char_uppercase(name: &String) -> String {
        let mut graphemes = name.graphemes(true);

        match graphemes.next() {
            None => String::new(),
            Some(c) => c.to_uppercase() + graphemes.as_str(),
        }
    }

    pub(crate) fn first_char_lowercase(name: &String) -> String {
        let mut graphemes = name.graphemes(true);

        match graphemes.next() {
            None => String::new(),
            Some(c) => c.to_lowercase() + graphemes.as_str(),
        }
    }

    pub(crate) fn as_type_name(name: &String) -> String {
        String::from("T") + Self::first_char_uppercase(name).as_str()
    }

    pub(crate) fn get_datatype_language_representation(datatype: &DataType) -> String {
        match datatype {
            DataType::Boolean => "Boolean".to_owned(),
            DataType::DateTime => "TDateTime".to_owned(),
            DataType::Date => "TDate".to_owned(),
            DataType::Double => "Double".to_owned(),
            DataType::Binary(_) => "TBytes".to_owned(),
            DataType::Integer => "TBytes".to_owned(),
            DataType::String => "String".to_owned(),
            DataType::Time => "TTime".to_owned(),
            DataType::Alias(a) => Self::as_type_name(a),
            DataType::Enumeration(e) => Self::as_type_name(e),
            DataType::Custom(c) => Self::as_type_name(c),
            DataType::List(s) => {
                let gt = Self::get_datatype_language_representation(s);

                format!("TList<{}>", gt)
            }
        }
    }
}
