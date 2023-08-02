use std::io::{BufWriter, Write};

use crate::generator::types::TypeAlias;

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
            buffer.write_fmt(format_args!(
                "{}T{} = {};\n",
                " ".repeat(indentation),
                type_alias.name,
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

    use crate::generator::types::DataType;

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

    // #[test]
    // fn write_all() {
    //     let type_aliases = vec![];
    //     let mut buffer = BufWriter::new(Vec::new());
    //     TypeAliasCodeGenerator::write_declarations(&mut buffer, &type_aliases, 0).unwrap();

    //     let bytes = buffer.into_inner().unwrap();
    //     let content = String::from_utf8(bytes).unwrap();

    //     let expected = indoc! {"
    //         {$REGION 'Helper'}
    //         function BinToHexStr(const pBin: TBytes): String;
    //         begin
    //           var vTemp: TBytes;
    //           BinToHex(pBin, 0, vTemp, Length(pBin));
    //           Result := TEncoding.GetString(vTemp);
    //         end;
    //         {$ENDREGION}
    //         \n"
    //     };

    //     assert_eq!(content, expected);
    // }
}
