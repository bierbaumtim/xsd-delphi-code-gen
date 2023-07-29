use std::{fs::File, io::Write};

use crate::generator::code_generator_trait::CodeGenOptions;

pub(crate) struct HelperCodeGenerator;

impl HelperCodeGenerator {
    pub(crate) fn write(
        file: &mut File,
        options: &CodeGenOptions,
        generate_date_time_helper: bool,
        generate_hex_binary_helper: bool,
    ) -> Result<(), std::io::Error> {
        if generate_date_time_helper || generate_hex_binary_helper {
            file.write_all(b"{$REGION 'Helper'}\n")?;

            if generate_date_time_helper {
                Self::write_date_time_helper(file, options)?;
            }

            if generate_date_time_helper && generate_hex_binary_helper {
                file.write_all(b"\n")?;
            }

            if generate_hex_binary_helper {
                Self::write_hex_binary_helper(file, options)?;
            }

            file.write_all(b"{$ENDREGION}\n")?;
            file.write_all(b"\n")?;
        }

        Ok(())
    }

    fn write_date_time_helper(
        file: &mut File,
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        if options.generate_from_xml {
            file
            .write_all(b"function DecodeDateTime(const pDateStr: String; const pFormat: String = ''): TDateTime;\n")?;
            file.write_all(b"begin\n")?;
            file.write_all(b"  if pFormat = '' then Exit(ISO8601ToDate(pDateStr));\n")?;
            file.write(b"\n")?;
            file.write_all(b"  Result := ISO8601ToDate(pDateStr);\n")?;
            file.write_all(b"end;\n")?;
            file.write_all(b"\n")?;

            file.write_all(
                b"function EncodeDateTime(const pDate: TDateTime; const pFormat: String = ''): String;\n",
            )?;
            file.write_all(b"begin\n")?;
            file.write_all(b"end;\n")?;
        }

        if options.generate_from_xml && options.generate_to_xml {
            file.write_all(b"\n")?;
        }

        if options.generate_to_xml {
            file.write_all(
                b"function EncodeTime(const pTime: TTime; const pFormat: String): String;\n",
            )?;
            file.write_all(b"begin\n")?;
            file.write_all(b"  var vFormatSettings := TFormatSettings.Create;\n")?;
            file.write_all(b"  vFormatSettings.LongTimeFormat := pFormat;\n")?;
            file.write_all(b"\n")?;
            file.write_all(b"  Result := TimeToStr(pTime, vFormatSettings);\n")?;
            file.write_all(b"end;\n")?;
        }

        Ok(())
    }

    fn write_hex_binary_helper(
        file: &mut File,
        options: &CodeGenOptions,
    ) -> Result<(), std::io::Error> {
        if options.generate_to_xml {
            file.write_all(b"function BinToHexStr(const pBin: TBytes): String;\n")?;
            file.write_all(b"begin\n")?;
            file.write_all(b"  var vTemp: TBytes;\n")?;
            file.write_all(b"  BinToHex(pBin, 0, vTemp, Length(pBin));\n")?;
            file.write_all(b"  Result := TEncoding.GetString(vTemp);\n")?;
            file.write_all(b"end;\n")?;
        }

        Ok(())
    }
}
