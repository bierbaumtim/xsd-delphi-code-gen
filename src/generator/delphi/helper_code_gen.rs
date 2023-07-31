use std::io::{BufWriter, Write};

use crate::generator::code_generator_trait::CodeGenOptions;

pub(crate) struct HelperCodeGenerator;

impl HelperCodeGenerator {
    pub(crate) fn write(
        buffer: &mut BufWriter<Box<dyn Write>>,
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

    fn write_date_time_helper(
        buffer: &mut BufWriter<Box<dyn Write>>,
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
            buffer.write_all(b"\n")?;

            buffer.write_all(
                b"function EncodeDateTime(const pDate: TDateTime; const pFormat: String = ''): String;\n",
            )?;
            buffer.write_all(b"begin\n")?;
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

    fn write_hex_binary_helper(
        buffer: &mut BufWriter<Box<dyn Write>>,
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
