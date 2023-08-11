use std::io::{BufWriter, Write};

use crate::generator::types::{DataType, TypeAlias};

use super::helper::Helper;

pub(crate) struct TypeAliasCodeGenerator;

impl TypeAliasCodeGenerator {
    pub(crate) fn write_declarations<T: Write>(
        buffer: &mut BufWriter<T>,
        type_aliases: &[TypeAlias],
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        if type_aliases.is_empty() {
            return Ok(());
        }

        buffer.write_fmt(format_args!(
            "{}{{$REGION 'Aliases'}}\n",
            " ".repeat(indentation),
        ))?;
        for type_alias in type_aliases {
            if matches!(&type_alias.for_type, DataType::FixedSizeList(_, _)) {
                continue;
            }

            buffer.write_fmt(format_args!(
                "{}{} = {};\n",
                " ".repeat(indentation),
                Helper::as_type_name(&type_alias.name),
                Helper::get_datatype_language_representation(&type_alias.for_type),
            ))?;
        }
        buffer.write_fmt(format_args!("{}{{$ENDREGION}}\n", " ".repeat(indentation)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use std::io::BufWriter;

    use crate::generator::types::{BinaryEncoding, DataType};

    use super::*;

    #[test]
    fn write_nothing_when_no_alias_available() {
        let type_aliases = vec![];
        let mut buffer = BufWriter::new(Vec::new());
        TypeAliasCodeGenerator::write_declarations(&mut buffer, &type_aliases, 0).unwrap();

        let bytes = buffer.into_inner().unwrap();
        let content = String::from_utf8(bytes).unwrap();

        assert_eq!(content, "");
    }

    #[test]
    fn write_with_0_indentation() {
        let type_aliases = vec![TypeAlias {
            pattern: None,
            name: String::from("CustomString"),
            for_type: DataType::String,
        }];
        let mut buffer = BufWriter::new(Vec::new());
        TypeAliasCodeGenerator::write_declarations(&mut buffer, &type_aliases, 0).unwrap();

        let bytes = buffer.into_inner().unwrap();
        let content = String::from_utf8(bytes).unwrap();

        let expected = indoc! {"
            {$REGION 'Aliases'}
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
            for_type: DataType::String,
        }];
        let mut buffer = BufWriter::new(Vec::new());
        TypeAliasCodeGenerator::write_declarations(&mut buffer, &type_aliases, 0).unwrap();

        let bytes = buffer.into_inner().unwrap();
        let content = String::from_utf8(bytes).unwrap();

        let expected = indoc! {"
              {$REGION 'Aliases'}
              TCustomString = String;
              {$ENDREGION}
            "
        };

        assert_eq!(content, expected);
    }

    #[test]
    fn write_all() {
        let type_aliases = vec![
            TypeAlias {
                pattern: None,
                name: String::from("CustomBool"),
                for_type: DataType::Boolean,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomDateTime"),
                for_type: DataType::DateTime,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomDate"),
                for_type: DataType::Date,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomDouble"),
                for_type: DataType::Double,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomBinary"),
                for_type: DataType::Binary(BinaryEncoding::Hex),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomShortInt"),
                for_type: DataType::ShortInteger,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomSmallInt"),
                for_type: DataType::SmallInteger,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomInteger"),
                for_type: DataType::Integer,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomLongInt"),
                for_type: DataType::LongInteger,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomShortUInt"),
                for_type: DataType::UnsignedShortInteger,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomSmallUInt"),
                for_type: DataType::UnsignedSmallInteger,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomUInt"),
                for_type: DataType::UnsignedInteger,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomLongUInt"),
                for_type: DataType::UnsignedLongInteger,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomString"),
                for_type: DataType::String,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomTime"),
                for_type: DataType::Time,
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomAlias"),
                for_type: DataType::Alias(String::from("NestedAlias")),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomEnum"),
                for_type: DataType::Enumeration(String::from("NestedEnum")),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomIntList"),
                for_type: DataType::List(Box::new(DataType::Integer)),
            },
            TypeAlias {
                pattern: None,
                name: String::from("CustomIntFixedList"),
                for_type: DataType::FixedSizeList(Box::new(DataType::Integer), 5),
            },
        ];
        let mut buffer = BufWriter::new(Vec::new());
        TypeAliasCodeGenerator::write_declarations(&mut buffer, &type_aliases, 0).unwrap();

        let bytes = buffer.into_inner().unwrap();
        let content = String::from_utf8(bytes).unwrap();

        let expected = indoc! {"
            {$REGION 'Aliases'}
            TCustomBool = Boolean;
            TCustomDateTime = TDateTime;
            TCustomDate = TDate;
            TCustomDouble = Double;
            TCustomBinary = TBytes;
            TCustomShortInt = ShortInt;
            TCustomSmallInt = SmallInt;
            TCustomInteger = Integer;
            TCustomLongInt = LongInt;
            TCustomShortUInt = Byte;
            TCustomSmallUInt = Word;
            TCustomUInt = NativeUInt;
            TCustomLongUInt = UInt64;
            TCustomString = String;
            TCustomTime = TTime;
            TCustomAlias = TNestedAlias;
            TCustomEnum = TNestedEnum;
            TCustomIntList = TList<Integer>;
            {$ENDREGION}
            "
        };

        assert_eq!(content, expected);
    }
}
