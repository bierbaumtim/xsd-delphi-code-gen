use std::io::Write;

use crate::generator::{
    code_generator_trait::CodeGenOptions,
    types::{DataType, TypeAlias},
};

use super::{code_writer::CodeWriter, helper::Helper};

/// Code generator for type aliases
///
/// # Example
///
/// ## Input
///
/// ```rust
/// use xsd_types::generator::types::{DataType, TypeAlias};
///
/// let type_aliases = vec![
///     TypeAlias {
///         pattern: None,
///         name: String::from("CustomString"),
///         qualified_name: String::from("CustomString"),
///         for_type: DataType::String,
///         documentations: Vec::new(),
///     },
///     TypeAlias {
///         pattern: None,
///         name: String::from("CustomIntList"),
///         qualified_name: String::from("CustomIntList"),
///         for_type: DataType::List(Box::new(DataType::Integer)),
///         documentations: Vec::new(),
///     },
/// ];
/// ```
///
/// ## Output
///
/// ```pascal
/// {$REGION 'Aliases'}
/// // XML Qualified Name: CustomString
/// TCustomString = String;
/// // XML Qualified Name: CustomIntList
/// TCustomIntList = TList<Integer>;
/// {$ENDREGION}
/// ```
pub struct TypeAliasCodeGenerator;

impl TypeAliasCodeGenerator {
    /// Writes the type aliases to the given writer
    /// 
    /// # Arguments
    /// 
    /// * `writer` - The writer to write the type aliases to
    /// * `type_aliases` - The type aliases to write
    /// * `options` - The code generation options
    /// * `indentation` - The indentation level
    pub(crate) fn write_declarations<T: Write>(
        writer: &mut CodeWriter<T>,
        type_aliases: &[TypeAlias],
        options: &CodeGenOptions,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        if type_aliases.is_empty() {
            return Ok(());
        }

        writer.writeln("{$REGION 'Aliases'}", Some(indentation))?;
        for type_alias in type_aliases {
            if matches!(&type_alias.for_type, DataType::FixedSizeList(_, _)) {
                continue;
            }

            writer.write_documentation(&type_alias.documentations, Some(indentation))?;
            Helper::write_qualified_name_comment(
                writer,
                &type_alias.qualified_name,
                Some(indentation),
            )?;

            writer.writeln_fmt(
                format_args!(
                    "{} = {};",
                    Helper::as_type_name(&type_alias.name, &options.type_prefix),
                    Helper::get_datatype_language_representation(
                        &type_alias.for_type,
                        &options.type_prefix
                    ),
                ),
                Some(indentation),
            )?;
        }
        writer.writeln("{$ENDREGION}", Some(indentation))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::io::BufWriter;

    use crate::generator::types::{BinaryEncoding, DataType};

    use super::*;

    #[test]
    fn write_nothing_when_no_alias_available() {
        let type_aliases = vec![];
        let options = CodeGenOptions::default();
        let buffer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer };
        TypeAliasCodeGenerator::write_declarations(&mut writer, &type_aliases, &options, 0)
            .unwrap();

        let bytes = writer.get_writer().unwrap().clone();
        let content = String::from_utf8(bytes).unwrap();

        assert_eq!(content, "");
    }

    #[test]
    fn write_with_0_indentation() {
        let type_aliases = vec![TypeAlias {
            pattern: None,
            name: String::from("CustomString"),
            qualified_name: String::from("CustomString"),
            for_type: DataType::String,
            documentations: Vec::new(),
        }];
        let options = CodeGenOptions::default();
        let buffer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer };
        TypeAliasCodeGenerator::write_declarations(&mut writer, &type_aliases, &options, 0)
            .unwrap();

        let bytes = writer.get_writer().unwrap().clone();
        let content = String::from_utf8(bytes).unwrap();

        let expected = indoc! {"
            {$REGION 'Aliases'}
            // XML Qualified Name: CustomString
            TCustomString = String;
            {$ENDREGION}
            "
        };

        assert_eq!(content, expected);
    }

    #[test]
    fn write_with_2_indentation() {
        let type_aliases = vec![TypeAlias {
            pattern: None,
            name: String::from("CustomString"),
            qualified_name: String::from("CustomString"),
            for_type: DataType::String,
            documentations: Vec::new(),
        }];
        let options = CodeGenOptions::default();
        let buffer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer };
        TypeAliasCodeGenerator::write_declarations(&mut writer, &type_aliases, &options, 2)
            .unwrap();

        let bytes = writer.get_writer().unwrap().clone();
        let content = String::from_utf8(bytes).unwrap();

        let expected = "  {$REGION 'Aliases'}\n  \
               // XML Qualified Name: CustomString\n  \
               TCustomString = String;\n  \
               {$ENDREGION}\n";

        assert_eq!(content, expected);
    }

    #[test]
    fn write_all() {
        let type_aliases = vec![
            TypeAlias {
                pattern: None,
                name: String::from("CustomBool"),
                qualified_name: String::from("CustomBool"),
                for_type: DataType::Boolean,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomDateTime"),
                qualified_name: String::from("CustomDateTime"),
                for_type: DataType::DateTime,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomDate"),
                qualified_name: String::from("CustomDate"),
                for_type: DataType::Date,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomDouble"),
                qualified_name: String::from("CustomDouble"),
                for_type: DataType::Double,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomBinary"),
                qualified_name: String::from("CustomBinary"),
                for_type: DataType::Binary(BinaryEncoding::Hex),
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomShortInt"),
                qualified_name: String::from("CustomShortInt"),
                for_type: DataType::ShortInteger,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomSmallInt"),
                qualified_name: String::from("CustomSmallInt"),
                for_type: DataType::SmallInteger,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomInteger"),
                qualified_name: String::from("CustomInteger"),
                for_type: DataType::Integer,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomLongInt"),
                qualified_name: String::from("CustomLongInt"),
                for_type: DataType::LongInteger,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomShortUInt"),
                qualified_name: String::from("CustomShortUInt"),
                for_type: DataType::UnsignedShortInteger,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomSmallUInt"),
                qualified_name: String::from("CustomSmallUInt"),
                for_type: DataType::UnsignedSmallInteger,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomUInt"),
                qualified_name: String::from("CustomUInt"),
                for_type: DataType::UnsignedInteger,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomLongUInt"),
                qualified_name: String::from("CustomLongUInt"),
                for_type: DataType::UnsignedLongInteger,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomString"),
                qualified_name: String::from("CustomString"),
                for_type: DataType::String,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomTime"),
                qualified_name: String::from("CustomTime"),
                for_type: DataType::Time,
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomAlias"),
                qualified_name: String::from("CustomAlias"),
                for_type: DataType::Alias(String::from("NestedAlias")),
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomEnum"),
                qualified_name: String::from("CustomEnum"),
                for_type: DataType::Enumeration(String::from("NestedEnum")),
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomIntList"),
                qualified_name: String::from("CustomIntList"),
                for_type: DataType::List(Box::new(DataType::Integer)),
                documentations: Vec::new(),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomIntFixedList"),
                qualified_name: String::from("CustomIntFixedList"),
                for_type: DataType::FixedSizeList(Box::new(DataType::Integer), 5),
                documentations: Vec::new(),
            },
        ];
        let options = CodeGenOptions::default();
        let buffer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer };
        TypeAliasCodeGenerator::write_declarations(&mut writer, &type_aliases, &options, 0)
            .unwrap();

        let bytes = writer.get_writer().unwrap().clone();
        let content = String::from_utf8(bytes).unwrap();

        let expected = indoc! {"
            {$REGION 'Aliases'}
            // XML Qualified Name: CustomBool
            TCustomBool = Boolean;
            // XML Qualified Name: CustomDateTime
            TCustomDateTime = TDateTime;
            // XML Qualified Name: CustomDate
            TCustomDate = TDate;
            // XML Qualified Name: CustomDouble
            TCustomDouble = Double;
            // XML Qualified Name: CustomBinary
            TCustomBinary = TBytes;
            // XML Qualified Name: CustomShortInt
            TCustomShortInt = ShortInt;
            // XML Qualified Name: CustomSmallInt
            TCustomSmallInt = SmallInt;
            // XML Qualified Name: CustomInteger
            TCustomInteger = Integer;
            // XML Qualified Name: CustomLongInt
            TCustomLongInt = LongInt;
            // XML Qualified Name: CustomShortUInt
            TCustomShortUInt = Byte;
            // XML Qualified Name: CustomSmallUInt
            TCustomSmallUInt = Word;
            // XML Qualified Name: CustomUInt
            TCustomUInt = NativeUInt;
            // XML Qualified Name: CustomLongUInt
            TCustomLongUInt = UInt64;
            // XML Qualified Name: CustomString
            TCustomString = String;
            // XML Qualified Name: CustomTime
            TCustomTime = TTime;
            // XML Qualified Name: CustomAlias
            TCustomAlias = TNestedAlias;
            // XML Qualified Name: CustomEnum
            TCustomEnum = TNestedEnum;
            // XML Qualified Name: CustomIntList
            TCustomIntList = TList<Integer>;
            {$ENDREGION}
            "
        };

        assert_eq!(content, expected);
    }
}
