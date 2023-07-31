use unicode_segmentation::UnicodeSegmentation;

use crate::generator::types::DataType;

pub(crate) struct Helper;

impl Helper {
    #[inline]
    pub(crate) fn first_char_uppercase(name: &String) -> String {
        let mut graphemes = name.graphemes(true);

        match graphemes.next() {
            None => String::new(),
            Some(c) => c.to_uppercase() + graphemes.as_str(),
        }
    }

    #[inline]
    pub(crate) fn first_char_lowercase(name: &String) -> String {
        let mut graphemes = name.graphemes(true);

        match graphemes.next() {
            None => String::new(),
            Some(c) => c.to_lowercase() + graphemes.as_str(),
        }
    }

    #[inline]
    pub(crate) fn as_type_name(name: &String) -> String {
        let mut result = String::with_capacity(name.len() + 1);
        result.push('T');
        result.push_str(&Self::first_char_uppercase(name));

        result
    }

    pub(crate) fn get_datatype_language_representation(datatype: &DataType) -> String {
        match datatype {
            DataType::Boolean => String::from("Boolean"),
            DataType::DateTime => String::from("TDateTime"),
            DataType::Date => String::from("TDate"),
            DataType::Double => String::from("Double"),
            DataType::Binary(_) => String::from("TBytes"),
            DataType::Integer => String::from("TBytes"),
            DataType::String => String::from("String"),
            DataType::Time => String::from("TTime"),
            DataType::Alias(a) => Self::as_type_name(a),
            DataType::Enumeration(e) => Self::as_type_name(e),
            DataType::Custom(c) => Self::as_type_name(c),
            DataType::FixedSizeList(t, _) => Self::get_datatype_language_representation(t),
            DataType::List(s) => {
                let gt = Self::get_datatype_language_representation(s);

                match **s {
                    DataType::Custom(_) => format!("TObjectList<{}>", gt),
                    _ => format!("TList<{}>", gt),
                }
            }
        }
    }
}
