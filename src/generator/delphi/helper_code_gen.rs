use std::io::Write;

use crate::generator::code_generator_trait::CodeGenOptions;

use super::code_writer::CodeWriter;

pub struct HelperCodeGenerator;

impl HelperCodeGenerator {
    pub fn write<T: Write>(
        writer: &mut CodeWriter<T>,
        options: &CodeGenOptions,
        generate_date_time_helper: bool,
        generate_hex_binary_helper: bool,
    ) -> Result<(), std::io::Error> {
        if generate_date_time_helper || generate_hex_binary_helper {
            writer.writeln("{$REGION 'Helper'}", None)?;

            if generate_date_time_helper {
                Self::write_date_time_helper(writer, options)?;
            }

            if generate_date_time_helper && generate_hex_binary_helper {
                writer.newline()?;
            }

            if generate_hex_binary_helper {
                Self::write_hex_binary_helper(writer, options)?;
            }

            writer.writeln("{$ENDREGION}", None)?;
            writer.newline()?;
        }

        Ok(())
    }

    fn write_date_time_helper<T: Write>(
        writer: &mut CodeWriter<T>,
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        if options.generate_from_xml {
            writer
            .writeln("function DecodeDateTime(const pDateStr: String; const pFormat: String = ''): TDateTime;", None)?;
            writer.writeln("begin", None)?;
            writer.writeln(
                "if pFormat = '' then Exit(ISO8601ToDate(pDateStr));",
                Some(2),
            )?;
            writer.newline()?;
            writer.writeln("Result := ISO8601ToDate(pDateStr);", Some(2))?;
            writer.writeln("end;", None)?;
        }

        if options.generate_from_xml && options.generate_to_xml {
            writer.newline()?;
        }

        if options.generate_to_xml {
            writer.writeln(
                "function EncodeTime(const pTime: TTime; const pFormat: String): String;",
                None,
            )?;
            writer.writeln("begin", None)?;
            writer.writeln("var vFormatSettings := TFormatSettings.Create;", Some(2))?;
            writer.writeln("vFormatSettings.LongTimeFormat := pFormat;", Some(2))?;
            writer.newline()?;
            writer.writeln("Result := TimeToStr(pTime, vFormatSettings);", Some(2))?;
            writer.writeln("end;", None)?;
        }

        Ok(())
    }

    fn write_hex_binary_helper<T: Write>(
        writer: &mut CodeWriter<T>,
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        if options.generate_to_xml {
            writer.writeln("function BinToHexStr(const pBin: TBytes): String;", None)?;
            writer.writeln("begin", None)?;
            writer.writeln("var vTemp: TBytes;", Some(2))?;
            writer.writeln("BinToHex(pBin, 0, vTemp, Length(pBin));", Some(2))?;
            writer.writeln("Result := TEncoding.GetString(vTemp);", Some(2))?;
            writer.writeln("end;", None)?;
        }

        if options.generate_from_xml && options.generate_to_xml {
            writer.newline()?;
        }

        if options.generate_from_xml {
            writer.writeln("function HexStrToBin(const pHex: String): TBytes;", None)?;
            writer.writeln("begin", None)?;
            writer.writeln("HexToBin(pHex, 0, Result, 0, Length(pHex) / 2);", Some(2))?;
            writer.writeln("end;", None)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::io::BufWriter;

    use crate::generator::code_generator_trait::CodeGenOptions;

    use super::*;

    #[test]
    fn write_nothing_when_all_flags_false() {
        let options = CodeGenOptions::default();
        let writer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer: writer };
        HelperCodeGenerator::write(&mut writer, &options, false, false).unwrap();

        let bytes = writer.get_writer().unwrap().clone();
        let content = String::from_utf8(bytes).unwrap();

        assert_eq!(content, "", "Content should be empty");
    }

    #[test]
    fn write_only_hex_binary_helper_when_from_xml_enabled() {
        let options = CodeGenOptions {
            generate_from_xml: false,
            generate_to_xml: true,
            unit_name: String::new(),
            type_prefix: None,
        };
        let writer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer: writer };
        HelperCodeGenerator::write(&mut writer, &options, false, true).unwrap();

        let bytes = writer.get_writer().unwrap().clone();
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
            type_prefix: None,
        };
        let writer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer: writer };
        HelperCodeGenerator::write(&mut writer, &options, false, true).unwrap();

        let bytes = writer.get_writer().unwrap().clone();
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
            type_prefix: None,
        };
        let writer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer: writer };
        HelperCodeGenerator::write(&mut writer, &options, true, false).unwrap();

        let bytes = writer.get_writer().unwrap().clone();
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
    fn writeln_helpers() {
        let options = CodeGenOptions {
            generate_from_xml: true,
            generate_to_xml: true,
            unit_name: String::new(),
            type_prefix: None,
        };
        let writer = BufWriter::new(Vec::new());
        let mut writer = CodeWriter { buffer: writer };
        HelperCodeGenerator::write(&mut writer, &options, true, true).unwrap();

        let bytes = writer.get_writer().unwrap().clone();
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
