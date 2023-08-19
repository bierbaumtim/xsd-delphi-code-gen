use unicode_segmentation::UnicodeSegmentation;

use crate::generator::types::{BinaryEncoding, DataType, TypeAlias};

pub(crate) struct Helper;

impl Helper {
    #[rustfmt::skip]
    const DELPHI_KEYWORDS: [&'static str; 66] = [
        "and", "array", "as", "asm", "automated", "begin", "case", "class", "const", "constructor", "destructor", "dispinterface",
        "div", "do", "downto", "else", "end", "except", "exports", "file", "finalization", "finally", "for", "function", "goto", "if", 
        "implementation", "in", "inherited", "initialization", "inline", "interface", "is", "label", "library", "mod", "nil", "not", 
        "object", "of", "or", "out", "packed", "procedure", "program", "property", "raise", "record", "repeat", "resourcestring",
        "set", "shl", "shr", "string", "then", "threadvar", "to", "try", "type", "unit", "until", "uses", "var", "while", "with", "xor",
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
    pub(crate) fn as_type_name(name: &String, prefix: &Option<String>) -> String {
        if name.is_empty() {
            return String::new();
        }

        let mut result =
            String::with_capacity(name.len() + prefix.as_ref().map_or(0, |p| p.len()) + 1);
        result.push('T');
        if let Some(prefix) = prefix {
            result.push_str(prefix.as_str());
        }
        result.push_str(&Self::first_char_uppercase(name));

        result
    }

    #[inline]
    pub(crate) fn as_variable_name(name: &str) -> String {
        if name.is_empty() {
            return String::new();
        }

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

    pub fn get_enum_variant_prefix(name: &str) -> String {
        let prefix = name
            .chars()
            .enumerate()
            .filter(|(i, c)| i == &0 || c.is_uppercase())
            .map(|(_, c)| c.to_ascii_lowercase())
            .collect::<Vec<char>>();

        String::from_iter(prefix)
    }

    pub(crate) fn get_datatype_language_representation(
        datatype: &DataType,
        prefix: &Option<String>,
    ) -> String {
        match datatype {
            DataType::Boolean => String::from("Boolean"),
            DataType::DateTime => String::from("TDateTime"),
            DataType::Date => String::from("TDate"),
            DataType::Double => String::from("Double"),
            DataType::Binary(_) => String::from("TBytes"),
            DataType::String => String::from("String"),
            DataType::Time => String::from("TTime"),
            DataType::Alias(a) => Self::as_type_name(a, prefix),
            DataType::Enumeration(e) => Self::as_type_name(e, prefix),
            DataType::Custom(c) => Self::as_type_name(c, prefix),
            DataType::FixedSizeList(t, _) => Self::get_datatype_language_representation(t, prefix),
            DataType::Union(u) => Self::as_type_name(u, prefix),
            DataType::List(s) => {
                let gt = Self::get_datatype_language_representation(s, prefix);

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

    pub(crate) fn get_variable_value_as_string(
        data_type: &DataType,
        variable_name: &String,
        pattern: Option<String>,
    ) -> String {
        match data_type {
            DataType::Boolean => {
                format!("IfThen({}, cnXmlTrueValue, cnXmlFalseValue)", variable_name)
            }
            DataType::DateTime | DataType::Date if pattern.is_some() => format!(
                "FormatDateTime('{}', {})",
                pattern.unwrap_or_default(),
                variable_name,
            ),
            DataType::DateTime | DataType::Date => format!("DateToISO8601({})", variable_name),
            DataType::Double => format!("FloatToStr({})", variable_name,),
            DataType::Binary(BinaryEncoding::Base64) => {
                format!("TNetEncoding.Base64.EncodeStringToBytes({})", variable_name,)
            }
            DataType::Binary(BinaryEncoding::Hex) => format!("BinToHexStr({})", variable_name,),
            DataType::String => variable_name.to_string(),
            DataType::Time if pattern.is_some() => format!(
                "EncodeTime({}, '{}')",
                variable_name,
                pattern.unwrap_or_default(),
            ),
            DataType::Time => format!("TimeToStr({})", variable_name,),
            DataType::SmallInteger
            | DataType::ShortInteger
            | DataType::Integer
            | DataType::LongInteger
            | DataType::UnsignedSmallInteger
            | DataType::UnsignedShortInteger
            | DataType::UnsignedInteger
            | DataType::UnsignedLongInteger => format!("IntToStr({})", variable_name),
            _ => "''".to_owned(),
        }
    }

    pub(crate) fn get_alias_data_type(
        alias: &str,
        type_aliases: &[TypeAlias],
    ) -> Option<(DataType, Option<String>)> {
        if let Some(t) = type_aliases.iter().find(|t| t.name == alias) {
            let mut pattern = t.pattern.clone();
            let mut data_type = t.for_type.clone();

            while let DataType::Custom(n) = &data_type {
                if let Some(alias) = type_aliases.iter().find(|t| t.name == n.as_str()) {
                    if pattern.is_none() {
                        pattern = alias.pattern.clone();
                    }

                    data_type = alias.for_type.clone();
                } else {
                    break;
                }
            }

            return Some((data_type, pattern));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

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
        let res = Helper::as_type_name(&String::new(), &None);

        assert_eq!(res, "");
    }

    #[test]
    fn as_type_name_with_nonempty_string() {
        let res = Helper::as_type_name(&String::from("SozialDaten"), &None);

        assert_eq!(res, "TSozialDaten");
    }

    #[test]
    fn as_variable_name_with_empty_string() {
        let res = Helper::as_variable_name(&String::new());

        assert_eq!(res, "");
    }

    #[test]
    fn as_variable_name_with_nonempty_string() {
        let res = Helper::as_variable_name(&"vorname".to_owned());

        assert_eq!(res, "Vorname");
    }

    #[test]
    fn as_variable_name_with_reserved_word() {
        let res = Helper::as_variable_name(&"label".to_owned());

        assert_eq!(res, "Label_");
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
            .map(|dt| Helper::get_datatype_language_representation(&dt, &None))
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
