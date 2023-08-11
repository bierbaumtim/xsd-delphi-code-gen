use unicode_segmentation::UnicodeSegmentation;

use crate::generator::types::DataType;

pub(crate) struct Helper;

impl Helper {
    const DELPHI_KEYWORDS: [&'static str; 66] = [
        "and",
        "array",
        "as",
        "asm",
        "automated",
        "begin",
        "case",
        "class",
        "const",
        "constructor",
        "destructor",
        "dispinterface",
        "div",
        "do",
        "downto",
        "else",
        "end",
        "except",
        "exports",
        "file",
        "finalization",
        "finally",
        "for",
        "function",
        "goto",
        "if",
        "implementation",
        "in",
        "inherited",
        "initialization",
        "inline",
        "interface",
        "is",
        "label",
        "library",
        "mod",
        "nil",
        "not",
        "object",
        "of",
        "or",
        "out",
        "packed",
        "procedure",
        "program",
        "property",
        "raise",
        "record",
        "repeat",
        "resourcestring",
        "set",
        "shl",
        "shr",
        "string",
        "then",
        "threadvar",
        "to",
        "try",
        "type",
        "unit",
        "until",
        "uses",
        "var",
        "while",
        "with",
        "xor",
    ];

    #[inline]
    pub(crate) fn first_char_uppercase(name: &String) -> String {
        let mut graphemes = name.graphemes(true);

        match graphemes.next() {
            None => String::new(),
            Some(c) => c.to_uppercase() + graphemes.as_str(),
        }
    }

    #[allow(dead_code)]
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
        if name.is_empty() {
            return String::new();
        }

        let mut result = String::with_capacity(name.len() + 1);
        result.push('T');
        result.push_str(&Self::first_char_uppercase(name));

        result
    }

    #[inline]
    pub(crate) fn as_variable_name(name: &str) -> String {
        let name = Self::sanitize_name(name);

        Self::first_char_uppercase(&name)
    }

    pub fn sanitize_name(name: &str) -> String {
        if Self::DELPHI_KEYWORDS
            .binary_search(&name.to_lowercase().as_str())
            .is_ok()
        {
            let mut name = name.to_owned();

            name.push('_');

            name
        } else {
            name.to_owned()
        }
    }

    pub(crate) fn get_datatype_language_representation(datatype: &DataType) -> String {
        match datatype {
            DataType::Boolean => String::from("Boolean"),
            DataType::DateTime => String::from("TDateTime"),
            DataType::Date => String::from("TDate"),
            DataType::Double => String::from("Double"),
            DataType::Binary(_) => String::from("TBytes"),
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
            DataType::ShortInteger => String::from("ShortInt"),
            DataType::SmallInteger => String::from("SmallInt"),
            DataType::Integer => String::from("Integer"),
            DataType::LongInteger => String::from("LongInt"),
            DataType::UnsignedShortInteger => String::from("Byte"),
            DataType::UnsignedSmallInteger => String::from("Word"),
            DataType::UnsignedInteger => String::from("NativeUInt"),
            DataType::UnsignedLongInteger => String::from("UInt64"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::types::BinaryEncoding;

    use super::*;

    #[test]
    fn first_char_uppercase_with_empty_string() {
        let res = Helper::first_char_uppercase(&String::new());

        assert_eq!(res, "");
    }

    #[test]
    fn first_char_uppercase_with_nonempty_string() {
        let res = Helper::first_char_uppercase(&String::from("test"));

        assert_eq!(res, "Test");
    }

    #[test]
    fn first_char_lowercase_with_empty_string() {
        let res = Helper::first_char_lowercase(&String::new());

        assert_eq!(res, "");
    }

    #[test]
    fn first_char_lowercase_with_nonempty_string() {
        let res = Helper::first_char_lowercase(&String::from("TEST"));

        assert_eq!(res, "tEST");
    }

    #[test]
    fn as_type_name_with_empty_string() {
        let res = Helper::as_type_name(&String::new());

        assert_eq!(res, "");
    }

    #[test]
    fn as_type_name_with_nonempty_string() {
        let res = Helper::as_type_name(&String::from("SozialDaten"));

        assert_eq!(res, "TSozialDaten");
    }

    #[test]
    fn get_datatype_language_representation() {
        let types = vec![
            DataType::Boolean,
            DataType::DateTime,
            DataType::Date,
            DataType::Double,
            DataType::Binary(BinaryEncoding::Base64),
            DataType::Binary(BinaryEncoding::Hex),
            DataType::String,
            DataType::Time,
            DataType::Alias(String::from("CustomAlias")),
            DataType::Enumeration(String::from("CustomEnum")),
            DataType::Custom(String::from("CustomClass")),
            DataType::FixedSizeList(Box::new(DataType::String), 1),
            DataType::List(Box::new(DataType::Integer)),
            DataType::List(Box::new(DataType::Custom(String::from("CustomListType")))),
            DataType::ShortInteger,
            DataType::SmallInteger,
            DataType::Integer,
            DataType::LongInteger,
            DataType::UnsignedShortInteger,
            DataType::UnsignedSmallInteger,
            DataType::UnsignedInteger,
            DataType::UnsignedLongInteger,
        ];

        let lr = types
            .into_iter()
            .map(|dt| Helper::get_datatype_language_representation(&dt))
            .collect::<Vec<String>>();

        let expected = vec![
            String::from("Boolean"),
            String::from("TDateTime"),
            String::from("TDate"),
            String::from("Double"),
            String::from("TBytes"),
            String::from("TBytes"),
            String::from("String"),
            String::from("TTime"),
            String::from("TCustomAlias"),
            String::from("TCustomEnum"),
            String::from("TCustomClass"),
            String::from("String"),
            String::from("TList<Integer>"),
            String::from("TObjectList<TCustomListType>"),
            String::from("ShortInt"),
            String::from("SmallInt"),
            String::from("Integer"),
            String::from("LongInt"),
            String::from("Byte"),
            String::from("Word"),
            String::from("NativeUInt"),
            String::from("UInt64"),
        ];

        assert_eq!(lr, expected);
    }
}
