use std::{fs::File, io::Write};

pub(crate) struct HelperCodeGenerator;

impl HelperCodeGenerator {
    pub(crate) fn write_date_time_helper(file: &mut File) -> Result<(), std::io::Error> {
        file
        .write_all(b"function DecodeDateTime(const pDateStr: String; const pFormat: String = ''): TDateTime;\n")?;
        file.write_all(b"begin\n")?;
        file.write_all(b"  if pFormat = '' then Exit(ISO8601ToDate(pDateStr));\n")?;
        file.write(b"\n")?;
        file.write_all(b"  Result := ISO8601ToDate(pDateStr);\n")?;
        file.write_all(b"end;\n")?;
        file.write(b"\n")?;

        file.write_all(
            b"function EncodeDateTime(const pDate: TDateTime; const pFormat: String = ''): String;\n",
        )?;
        file.write_all(b"begin\n")?;
        file.write_all(b"end;\n")?;

        file.write_all(b"\n")?;

        file.write_all(b"function EncodeTime(const pTime: TTime; const pFormat: String): String;\n")?;
        file.write_all(b"begin\n")?;
        file.write_all(b"  var vFormatSettings := TFormatSettings.Create;\n")?;
        file.write_all(b"  vFormatSettings.LongTimeFormat := pFormat;\n")?;
        file.write_all(b"\n")?;
        file.write_all(b"  Result := TimeToStr(pTime, vFormatSettings);\n")?;
        file.write_all(b"end;\n")?;

        Ok(())
    }

    pub(crate) fn write_hex_binary_helper(file: &mut File) -> Result<(), std::io::Error> {
        file.write_all(b"function BinToHexStr(const pBin: TBytes): String;\n")?;
        file.write_all(b"begin\n")?;
        file.write_all(b"  var vTemp: TBytes;\n")?;
        file.write_all(b"  BinToHex(pBin, 0, vTemp, Length(pBin));\n")?;
        file.write_all(b"  Result := TEncoding.GetString(vTemp);\n")?;
        file.write_all(b"end;\n")?;

        Ok(())
    }
}
