use std::io::{BufWriter, Write};

use crate::generator::code_generator_trait::CodeGenOptions;

pub(crate) struct HelperCodeGenerator;

impl HelperCodeGenerator {
    pub(crate) fn write<T: Write>(
        buffer: &mut BufWriter<T>,
        options: &CodeGenOptions,
        generate_date_time_helper: bool,
        generate_hex_binary_helper: bool,
    ) -> Result<(), std::io::Error> {
        if generate_date_time_helper || generate_hex_binary_helper {
            buffer.write_all(b"{$REGION 'Helper'}\n")?;

            if generate_date_time_helper {
                Self::write_date_time_helper(buffer, options)?;
            }

            if generate_date_time_helper && generate_hex_binary_helper {
                buffer.write_all(b"\n")?;
            }

            if generate_hex_binary_helper {
                Self::write_hex_binary_helper(buffer, options)?;
            }

            buffer.write_all(b"{$ENDREGION}\n")?;
            buffer.write_all(b"\n")?;
        }

        Ok(())
    }

    fn write_date_time_helper<T: Write>(
        buffer: &mut BufWriter<T>,
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        if options.generate_from_xml {
            buffer
            .write_all(b"function DecodeDateTime(const pDateStr: String; const pFormat: String = ''): TDateTime;\n")?;
            buffer.write_all(b"begin\n")?;
            buffer.write_all(b"  if pFormat = '' then Exit(ISO8601ToDate(pDateStr));\n")?;
            buffer.write_all(b"\n")?;
            buffer.write_all(b"  Result := ISO8601ToDate(pDateStr);\n")?;
            buffer.write_all(b"end;\n")?;
        }

        if options.generate_from_xml && options.generate_to_xml {
            buffer.write_all(b"\n")?;
        }

        if options.generate_to_xml {
            buffer.write_all(
                b"function EncodeTime(const pTime: TTime; const pFormat: String): String;\n",
            )?;
            buffer.write_all(b"begin\n")?;
            buffer.write_all(b"  var vFormatSettings := TFormatSettings.Create;\n")?;
            buffer.write_all(b"  vFormatSettings.LongTimeFormat := pFormat;\n")?;
            buffer.write_all(b"\n")?;
            buffer.write_all(b"  Result := TimeToStr(pTime, vFormatSettings);\n")?;
            buffer.write_all(b"end;\n")?;
        }

        Ok(())
    }

    fn write_hex_binary_helper<T: Write>(
        buffer: &mut BufWriter<T>,
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        if options.generate_to_xml {
            buffer.write_all(b"function BinToHexStr(const pBin: TBytes): String;\n")?;
            buffer.write_all(b"begin\n")?;
            buffer.write_all(b"  var vTemp: TBytes;\n")?;
            buffer.write_all(b"  BinToHex(pBin, 0, vTemp, Length(pBin));\n")?;
            buffer.write_all(b"  Result := TEncoding.GetString(vTemp);\n")?;
            buffer.write_all(b"end;\n")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use std::io::BufWriter;

    use crate::generator::code_generator_trait::CodeGenOptions;

    use super::*;

    #[test]
    fn write_nothing_when_all_flags_false() {
        let options = CodeGenOptions::default();
        let mut buffer = BufWriter::new(Vec::new());
        HelperCodeGenerator::write(&mut buffer, &options, false, false).unwrap();

        let bytes = buffer.into_inner().unwrap();
        let content = String::from_utf8(bytes).unwrap();

        assert_eq!(content, "", "Content should be empty");
    }

    #[test]
    fn write_only_hex_binary_helper_when_from_xml_enabled() {
        let options = CodeGenOptions {
            generate_from_xml: false,
            generate_to_xml: true,
            unit_name: String::new(),
        };
        let mut buffer = BufWriter::new(Vec::new());
        HelperCodeGenerator::write(&mut buffer, &options, false, true).unwrap();

        let bytes = buffer.into_inner().unwrap();
        let content = String::from_utf8(bytes).unwrap();

        let expected = indoc! {"
            {$REGION 'Helper'}
            function BinToHexStr(const pBin: TBytes): String;
            begin
              var vTemp: TBytes;
              BinToHex(pBin, 0, vTemp, Length(pBin));
              Result := TEncoding.GetString(vTemp);
            end;
            {$ENDREGION}
            \n"
        };

        assert_eq!(content, expected);
    }

    #[test]
    fn write_only_hex_binary_helper_when_from_xml_disabled() {
        let options = CodeGenOptions {
            generate_from_xml: false,
            generate_to_xml: false,
            unit_name: String::new(),
        };
        let mut buffer = BufWriter::new(Vec::new());
        HelperCodeGenerator::write(&mut buffer, &options, false, true).unwrap();

        let bytes = buffer.into_inner().unwrap();
        let content = String::from_utf8(bytes).unwrap();

        let expected = indoc! {"
            {$REGION 'Helper'}
            {$ENDREGION}
            \n"
        };

        assert_eq!(content, expected);
    }

    #[test]
    fn write_date_time_helper_when_all_flags_enabled() {
        let options = CodeGenOptions {
            generate_from_xml: true,
            generate_to_xml: true,
            unit_name: String::new(),
        };
        let mut buffer = BufWriter::new(Vec::new());
        HelperCodeGenerator::write(&mut buffer, &options, true, false).unwrap();

        let bytes = buffer.into_inner().unwrap();
        let content = String::from_utf8(bytes).unwrap();

        let expected = indoc! {"
            {$REGION 'Helper'}
            function DecodeDateTime(const pDateStr: String; const pFormat: String = ''): TDateTime;
            begin
              if pFormat = '' then Exit(ISO8601ToDate(pDateStr));

              Result := ISO8601ToDate(pDateStr);
            end;

            function EncodeTime(const pTime: TTime; const pFormat: String): String;
            begin
              var vFormatSettings := TFormatSettings.Create;
              vFormatSettings.LongTimeFormat := pFormat;

              Result := TimeToStr(pTime, vFormatSettings);
            end;
            {$ENDREGION}
            \n"
        };

        assert_eq!(content, expected);
    }

    #[test]
    fn write_all_helpers() {
        let options = CodeGenOptions {
            generate_from_xml: true,
            generate_to_xml: true,
            unit_name: String::new(),
        };
        let mut buffer = BufWriter::new(Vec::new());
        HelperCodeGenerator::write(&mut buffer, &options, true, true).unwrap();

        let bytes = buffer.into_inner().unwrap();
        let content = String::from_utf8(bytes).unwrap();

        let expected = indoc! {"
            {$REGION 'Helper'}
            function DecodeDateTime(const pDateStr: String; const pFormat: String = ''): TDateTime;
            begin
              if pFormat = '' then Exit(ISO8601ToDate(pDateStr));

              Result := ISO8601ToDate(pDateStr);
            end;

            function EncodeTime(const pTime: TTime; const pFormat: String): String;
            begin
              var vFormatSettings := TFormatSettings.Create;
              vFormatSettings.LongTimeFormat := pFormat;

              Result := TimeToStr(pTime, vFormatSettings);
            end;

            function BinToHexStr(const pBin: TBytes): String;
            begin
              var vTemp: TBytes;
              BinToHex(pBin, 0, vTemp, Length(pBin));
              Result := TEncoding.GetString(vTemp);
            end;
            {$ENDREGION}
            \n"
        };

        assert_eq!(content, expected);
    }
}
