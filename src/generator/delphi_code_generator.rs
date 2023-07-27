use std::{fs::File, io::Write};

use unicode_segmentation::UnicodeSegmentation;

use super::code_generator_trait::CodeGenerator;
use super::internal_representation::{InternalRepresentation, DOCUMENT_NAME};
use crate::generator::types::*;

// TODO: No forward declaration for document
// TODO: build IR(Intermediate Representation) with more informations about DataType, Inheritance
// TODO: Sort Document class first
// TODO: Sort class Declarations by occurance in document, then by inheritance and dependency

pub(crate) struct DelphiCodeGenerator<'a> {
    file: &'a mut File,
    unit_name: String,
    internal_representation: InternalRepresentation,
    generate_date_time_helper: bool,
    generate_hex_binary_helper: bool,
}

impl<'a> DelphiCodeGenerator<'a> {
    fn write_unit(&mut self) -> Result<(), std::io::Error> {
        self.file
            .write_fmt(format_args!("unit {};", self.unit_name))?;
        self.newline()?;
        self.newline()
    }

    fn write_uses(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"uses System.DateUtils,\n")?;
        self.file.write_all(b"     System.Types,\n")?;
        self.file.write_all(b"     System.Xml;")?;
        self.newline()?;
        self.newline()
    }

    fn write_interface_start(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"interface")?;
        self.newline()?;
        self.newline()
    }

    fn write_forward_declerations(&mut self) -> Result<(), std::io::Error> {
        self.file.write(b"types")?;
        self.newline()?;
        self.newline()?;

        if !self.internal_representation.enumerations.is_empty() {
            self.file.write_all(b"  {$REGION 'Enumerations'}\n")?;
            for e in &self.internal_representation.enumerations {
                Self::generate_enumeration_declaration(e, self.file, 2)?;
            }
            self.file.write_all(b"  {$ENDREGION}\n")?;

            self.newline()?;
            self.file
                .write_all(b"  {$REGION 'Enumerations Helper'}\n")?;
            for e in &self.internal_representation.enumerations {
                Self::generate_enumeration_helper_declaration(e, self.file, 2)?;
            }
            self.file.write_all(b"  {$ENDREGION}\n")?;
            self.newline()?;
        }

        if !self.internal_representation.types_aliases.is_empty() {
            self.file.write_all(b"  {$REGION 'Aliases'}\n")?;
            for a in &self.internal_representation.types_aliases {
                Self::generate_type_alias_declaration(a, self.file, 2)?;
            }
            self.file.write_all(b"  {$ENDREGION}\n")?;
            self.newline()?;
        }

        if !self.internal_representation.classes.is_empty() {
            self.file
                .write_all(b"  {$REGION 'Forward Declarations}\n")?;
            for class_type in &self.internal_representation.classes {
                if class_type.name == DOCUMENT_NAME {
                    continue;
                }

                Self::generate_class_forward_declaration(class_type, self.file, 2)?;
            }
            self.file.write_all(b"  {$ENDREGION}\n")?;
            self.newline()?;
        }

        Ok(())
    }

    fn write_declarations(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"  {$REGION 'Declarations}\n")?;

        Self::generate_class_declaration(&self.internal_representation.document, self.file, 2)?;

        for class_type in &self.internal_representation.classes {
            if class_type.name == DOCUMENT_NAME {
                continue;
            }

            Self::generate_class_declaration(class_type, self.file, 2)?;
        }
        self.file.write_all(b"  {$ENDREGION}\n")?;
        self.newline()?;

        Ok(())
    }

    fn write_implementation_start(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"implementation")?;
        self.newline()?;
        self.newline()
    }

    fn write_implementation(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"{$REGION 'Enumerations Helper'}\n")?;
        for enumeration in &self.internal_representation.enumerations {
            Self::generate_enumeration_helper_implementation(enumeration, self.file)?;
        }
        self.file.write_all(b"{$ENDREGION}\n")?;

        self.newline()?;

        if self.generate_date_time_helper {
            // TODO: Anpassen
            self.file
                .write_all(b"function DecodeDateTime(const pDateStr: String; const pFormat: String = ''): TDateTime;\n")?;
            self.file.write_all(b"begin\n")?;
            self.file
                .write_all(b"  if pFormat = '' then Exit(ISO8601ToDate(pDateStr));\n")?;
            self.newline()?;
            self.file
                .write_all(b"  Result := ISO8601ToDate(pDateStr);")?;
            self.file.write_all(b"end;\n")?;
            self.newline()?;

            self.file
                .write_all(b"function EncodeDateTime(const pDate: TDateTime; const pFormat: String = ''): String;")?;
            self.file.write_all(b"begin\n")?;
            self.file.write_all(b"end;\n")?;

            self.newline()?;

            self.file.write_all(
                b"function EncodeTime(const pTime: TTime; const pFormat: String): String;",
            )?;
            self.file.write_all(b"begin\n")?;
            self.file
                .write_all(b"  var vFormatSettings := TFormatSettings.Create;\n")?;
            self.file
                .write_all(b"  vFormatSettings.LongTimeFormat := pFormat;\n")?;
            self.file
                .write_all(b"  Result := TimeToStr(pTime, vFormatSettings);\n")?;
            self.file.write_all(b"end;\n")?;

            self.newline()?;
        }

        if self.generate_hex_binary_helper {
            self.file
                .write_all(b"function BinToHexStr(const pBin: TBytes): String;\n")?;
            self.file.write_all(b"begin\n")?;
            self.file.write_all(b"  var vTemp: TBytes;\n")?;
            self.file
                .write_all(b"  BinToHex(pBin, 0, vTemp, Length(pBin));\n")?;
            self.file
                .write_all(b"  Result := TEncoding.GetString(vTemp);\n")?;
            self.file.write_all(b"end;\n")?;

            self.newline()?;
        }

        self.file.write_all(b"{$REGION 'Classes'}\n")?;
        // TODO: Generate Document Implementation

        self.newline()?;

        for class_type in &self.internal_representation.classes {
            Self::generate_class_implementation(
                class_type,
                self.file,
                &self.internal_representation,
            )?;
        }
        self.file.write_all(b"{$ENDREGION}\n")?;

        self.newline()?;

        Ok(())
    }

    fn write_file_end(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"end.")
    }

    fn newline(&mut self) -> Result<(), std::io::Error> {
        self.file.write_all(b"\n")
    }

    // Generator functions
    // TypeAlias
    fn generate_type_alias_declaration(
        type_alias: &TypeAlias,
        file: &mut File,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{} = {};\n",
            " ".repeat(indentation),
            type_alias.name,
            Self::get_datatype_language_representation(&type_alias.for_type),
        ))
    }

    // Enumeration
    fn generate_enumeration_declaration(
        enumeration: &Enumeration,
        file: &mut File,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{} = ({});\n",
            " ".repeat(indentation),
            Self::first_char_uppercase(&enumeration.name),
            enumeration
                .values
                .iter()
                .map(|v| Self::first_char_lowercase(&v.variant_name))
                .collect::<Vec<String>>()
                .join(", ")
        ))
    }

    fn generate_enumeration_helper_declaration(
        enumeration: &Enumeration,
        file: &mut File,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        let formatted_enum_name = Self::first_char_uppercase(&enumeration.name);

        file.write_fmt(format_args!(
            "{}T{}Helper = record helper for T{}\n",
            " ".repeat(indentation),
            formatted_enum_name,
            formatted_enum_name,
        ))?;
        file.write_fmt(format_args!(
            "{}class function FromXmlValue(const pXmlValue: String): T{}; static;\n",
            " ".repeat(indentation + 2),
            formatted_enum_name,
        ))?;
        file.write_fmt(format_args!(
            "{}function ToXmlValue: String;\n",
            " ".repeat(indentation + 2),
        ))?;
        file.write_fmt(format_args!("{}end;", " ".repeat(indentation),))?;
        file.write(b"\n")?;
        file.write(b"\n")?;

        Ok(())
    }

    fn generate_enumeration_helper_implementation(
        enumeration: &Enumeration,
        file: &mut File,
    ) -> Result<(), std::io::Error> {
        let formatted_enum_name = Self::first_char_uppercase(&enumeration.name);

        // Generate FromXmlValue
        let max_xml_value_len = enumeration
            .values
            .iter()
            .map(|v| v.xml_value.len() + 1)
            .max()
            .unwrap_or(4);

        file.write_fmt(format_args!(
            "class function T{}Helper.FromXmlValue(const pXmlValue: String): T{};\n",
            formatted_enum_name, formatted_enum_name,
        ))?;
        file.write_all(b"begin\n")?;
        file.write_all(b"  case pXmlValue of\n")?;

        for value in &enumeration.values {
            file.write_fmt(format_args!(
                "    '{}':{}Result := T{}.{};\n",
                value.xml_value,
                " ".repeat(max_xml_value_len - value.xml_value.len()),
                formatted_enum_name,
                Self::first_char_lowercase(&value.variant_name),
            ))?;
        }
        // file.write_all(b"    else Result := '';\n")?;
        file.write_all(b"  end;\n")?;

        file.write_all(b"end;\n")?;
        file.write_all(b"\n")?;

        // Generate ToXmlValue
        let max_variant_len = enumeration
            .values
            .iter()
            .map(|v| v.variant_name.len() + 1)
            .max()
            .unwrap_or(4);

        file.write_fmt(format_args!(
            "function T{}Helper.ToXmlValue: String;\n",
            formatted_enum_name,
        ))?;
        file.write_all(b"begin\n")?;
        file.write_all(b"  case Self of\n")?;

        for value in &enumeration.values {
            let formatted_variant_name = Self::first_char_lowercase(&value.variant_name);

            file.write_fmt(format_args!(
                "    {}:{}Result := '{}';\n",
                formatted_variant_name,
                " ".repeat(max_variant_len - value.variant_name.len()),
                formatted_variant_name
            ))?;
        }
        file.write_all(b"    else Result := '';\n")?;

        file.write_all(b"  end;\n")?;
        file.write_all(b"end;\n")?;
        file.write_all(b"\n")?;

        Ok(())
    }

    // ClassType
    fn generate_class_forward_declaration(
        class_type: &ClassType,
        file: &mut File,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{} = class;\n",
            " ".repeat(indentation),
            Self::first_char_uppercase(&class_type.name),
        ))
    }

    fn generate_class_declaration(
        class_type: &ClassType,
        file: &mut File,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}T{} = class{}",
            " ".repeat(indentation),
            Self::first_char_uppercase(&class_type.name),
            class_type.super_type.as_ref().map_or_else(
                || "(TObject)".to_owned(),
                |v| format!("(T{})", Self::first_char_uppercase(&v))
            )
        ))?;
        file.write_all(b"\n")?;
        file.write_fmt(format_args!("{}public\n", " ".repeat(indentation)))?;

        // constructors and destructors
        file.write_fmt(format_args!(
            "{}constructor FromXml(node: IXMLNode);\n",
            " ".repeat(indentation + 2),
        ))?;

        if class_type.variables.iter().any(|v| v.requires_free) {
            file.write_fmt(format_args!(
                "{}destructor Destroy; override;\n",
                " ".repeat(indentation + 2),
            ))?;
        }
        file.write_all(b"\n")?;
        file.write_fmt(format_args!(
            "{}procedure AppendToXmlRaw(pParent: IXMLNode);\n",
            " ".repeat(indentation + 2),
        ))?;
        // file.write_fmt(format_args!(
        //     "{}function ToXml: String;\n",
        //     " ".repeat(indentation + 2),
        // ))?;
        file.write_all(b"\n")?;

        // Variables
        for variable in &class_type.variables {
            Self::generate_variable_declaration(variable, file, indentation + 2)?;
        }

        file.write_fmt(format_args!("{}end;\n\n", " ".repeat(indentation)))?;

        Ok(())
    }

    fn generate_class_implementation(
        class_type: &ClassType,
        file: &mut File,
        internal_representation: &InternalRepresentation,
    ) -> Result<(), std::io::Error> {
        let formated_name = Self::as_type_name(&class_type.name);
        let needs_destroy = class_type.variables.iter().any(|v| v.requires_free);

        file.write_fmt(format_args!("{{ {} }}\n", formated_name))?;

        file.write_fmt(format_args!(
            "constructor {}.FromXml(node: IXMLNode);\n",
            formated_name,
        ))?;
        file.write(b"begin\n")?;

        for variable in &class_type.variables {
            match &variable.data_type {
                DataType::Boolean => {
                    file.write_fmt(format_args!(
                        "  {} := (node.ChildNodes['{}'].Text = 'true') or (node.ChildNodes['{}'].Text = '1');\n",
                        Self::first_char_uppercase(&variable.name),
                        variable.name, variable.name
                    ))?;
                }
                DataType::DateTime | DataType::Date => {
                    file.write_fmt(format_args!(
                        "  {} := ISO8601ToDate(node.ChildNodes['{}'].Text);\n",
                        Self::first_char_uppercase(&variable.name),
                        variable.name,
                    ))?;
                }
                DataType::Double => {
                    file.write_fmt(format_args!(
                        "  {} := StrToFloat(node.ChildNodes['{}'].Text);\n",
                        Self::first_char_uppercase(&variable.name),
                        variable.name
                    ))?;
                }
                DataType::Binary(BinaryEncoding::Base64) => {
                    file.write_fmt(format_args!(
                        "  {} := TNetEncoding.Base64.DecodeStringToBytes(node.ChildNodes['{}'].Text);\n",
                        Self::first_char_uppercase(&variable.name),
                        variable.name
                    ))?;
                }
                DataType::Binary(BinaryEncoding::Hex) => {
                    file.write_fmt(format_args!(
                        "  HexToBin(node.ChildNodes['{}'].Text, 0, {}, 0, Length(node.ChildNodes['{}'].Text));\n",
                        variable.name,
                        Self::first_char_uppercase(&variable.name),
                        variable.name,
                    ))?;
                }
                DataType::Integer => {
                    file.write_fmt(format_args!(
                        "  {} := StrToInt(node.ChildNodes['{}'].Text);\n",
                        Self::first_char_uppercase(&variable.name),
                        variable.name
                    ))?;
                }
                DataType::String => {
                    file.write_fmt(format_args!(
                        "  {} := node.ChildNodes['{}'].Text;\n",
                        Self::first_char_uppercase(&variable.name),
                        variable.name
                    ))?;
                }
                DataType::Time => {
                    file.write_fmt(format_args!(
                        "  {} := TimeOf(ISO8601ToDate(node.ChildNodes['{}'].Text));\n",
                        Self::first_char_uppercase(&variable.name),
                        variable.name
                    ))?;
                }
                DataType::Custom(name) => {
                    let type_alias = internal_representation
                        .types_aliases
                        .iter()
                        .find(|t| t.name == name.as_str());

                    match type_alias {
                        Some(t) => {
                            let mut pattern = t.pattern.clone();
                            let mut data_type = t.for_type.clone();

                            loop {
                                match &data_type {
                                    DataType::Custom(n) => {
                                        let type_alias = internal_representation
                                            .types_aliases
                                            .iter()
                                            .find(|t| t.name == n.as_str());

                                        if let Some(alias) = type_alias {
                                            if pattern.is_none() {
                                                pattern = alias.pattern.clone();
                                            }

                                            data_type = alias.for_type.clone();
                                        } else {
                                            break;
                                        }
                                    }
                                    _ => break,
                                }
                            }

                            match data_type {
                                DataType::Boolean => {
                                    file.write_fmt(format_args!(
                                        "  {} := (node.ChildNodes['{}'].Text = 'true') or (node.ChildNodes['{}'].Text = '1');\n",
                                        Self::first_char_uppercase(&variable.name),
                                        variable.name, variable.name
                                    ))?;
                                }
                                DataType::DateTime | DataType::Date if pattern.is_some() => {
                                    file.write_fmt(format_args!(
                                        "  {} := DecodeDateTime(node.ChildNodes['{}'].Text, '{}');\n",
                                        Self::first_char_uppercase(&variable.name),
                                        variable.name,
                                        pattern.unwrap_or_default(),
                                    ))?;
                                }
                                DataType::DateTime | DataType::Date => {
                                    file.write_fmt(format_args!(
                                        "  {} := ISO8601ToDate(node.ChildNodes['{}'].Text);\n",
                                        Self::first_char_uppercase(&variable.name),
                                        variable.name,
                                    ))?;
                                }
                                DataType::Double => {
                                    file.write_fmt(format_args!(
                                        "  {} := StrToFloat(node.ChildNodes['{}'].Text);\n",
                                        Self::first_char_uppercase(&variable.name),
                                        variable.name
                                    ))?;
                                }
                                DataType::Binary(BinaryEncoding::Base64) => {
                                    file.write_fmt(format_args!(
                                        "  {} := TNetEncoding.Base64.DecodeStringToBytes(node.ChildNodes['{}'].Text);\n",
                                        Self::first_char_uppercase(&variable.name),
                                        variable.name
                                    ))?;
                                }
                                DataType::Binary(BinaryEncoding::Hex) => {
                                    file.write_fmt(format_args!(
                                        "  HexToBin(node.ChildNodes['{}'].Text, 0, {}, 0, Length(node.ChildNodes['{}'].Text));\n",
                                        variable.name,
                                        Self::first_char_uppercase(&variable.name),
                                        variable.name,
                                    ))?;
                                }
                                DataType::Integer => {
                                    file.write_fmt(format_args!(
                                        "  {} := StrToInt(node.ChildNodes['{}'].Text);\n",
                                        Self::first_char_uppercase(&variable.name),
                                        variable.name
                                    ))?;
                                }
                                DataType::String => {
                                    file.write_fmt(format_args!(
                                        "  {} := node.ChildNodes['{}'].Text;\n",
                                        Self::first_char_uppercase(&variable.name),
                                        variable.name
                                    ))?;
                                }
                                DataType::Time if pattern.is_some() => {
                                    file.write_fmt(format_args!(
                                        "  {} := TimeOf(DecodeDateTime(node.ChildNodes['{}'].Text, '{}'));\n",
                                        Self::first_char_uppercase(&variable.name),
                                        variable.name,
                                        pattern.unwrap_or_default(),
                                    ))?;
                                }
                                DataType::Time => {
                                    file.write_fmt(format_args!(
                                        "  {} := TimeOf(ISO8601ToDate(node.ChildNodes['{}'].Text));\n",
                                        Self::first_char_uppercase(&variable.name),
                                        variable.name
                                    ))?;
                                }
                                _ => (),
                            }
                        }
                        None => {
                            file.write_fmt(format_args!(
                                "  {} := {}.FromXml(node.ChildNodes['{}']);\n",
                                Self::first_char_uppercase(&variable.name),
                                Self::as_type_name(name),
                                variable.name
                            ))?;
                        }
                    }
                }
                DataType::List(_) => todo!(),
            }
        }

        file.write(b"end;\n")?;

        if needs_destroy {
            file.write(b"\n")?;
            file.write_fmt(format_args!("destructor {}.Destroy;\n", formated_name))?;

            file.write(b"begin\n")?;

            for variable in class_type.variables.iter().filter(|v| v.requires_free) {
                file.write_fmt(format_args!(
                    "  {}.Free;\n",
                    Self::first_char_uppercase(&variable.name)
                ))?;
            }

            file.write_all(b"\n")?;
            file.write(b"  inherited;\n")?;
            file.write(b"end;\n")?;
        }

        file.write_all(b"\n")?;
        file.write_fmt(format_args!(
            "procedure {}.AppendToXmlRaw(pParent: IXMLNode);\n",
            formated_name
        ))?;

        file.write(b"begin\n")?;
        file.write(b"  var node: IXMLNode;\n")?;
        file.write_all(b"\n")?;
        for variable in &class_type.variables {
            match &variable.data_type {
                DataType::Boolean => {
                    file.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.name,
                    ))?;
                    file.write_fmt(format_args!(
                        "  node.Text := IfThen({}, 'true', 'false');\n",
                        Self::first_char_uppercase(&variable.name),
                    ))?;
                }
                DataType::DateTime | DataType::Date => {
                    file.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.name,
                    ))?;
                    file.write_fmt(format_args!(
                        "  node.Text := DateToISO8601({});\n",
                        Self::first_char_uppercase(&variable.name),
                    ))?;
                }
                DataType::Double => {
                    file.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.name,
                    ))?;
                    file.write_fmt(format_args!(
                        "  node.Text := FloatToStr({});\n",
                        Self::first_char_uppercase(&variable.name),
                    ))?;
                }
                DataType::Binary(BinaryEncoding::Base64) => {
                    file.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.name,
                    ))?;
                    file.write_fmt(format_args!(
                        "  node.Text := TNetEncoding.Base64.EncodeStringToBytes({});\n",
                        Self::first_char_uppercase(&variable.name),
                    ))?;
                }
                DataType::Binary(BinaryEncoding::Hex) => {
                    file.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.name,
                    ))?;
                    file.write_fmt(format_args!(
                        "  node.Text := BinToHexStr({});\n",
                        Self::first_char_uppercase(&variable.name),
                    ))?;
                }
                DataType::Integer => {
                    file.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.name,
                    ))?;
                    file.write_fmt(format_args!(
                        "  node.Text := IntToStr({});\n",
                        Self::first_char_uppercase(&variable.name),
                    ))?;
                }
                DataType::String => {
                    file.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.name,
                    ))?;
                    file.write_fmt(format_args!(
                        "  node.Text := {};\n",
                        Self::first_char_uppercase(&variable.name),
                    ))?;
                }
                DataType::Time => {
                    file.write_fmt(format_args!(
                        "  node := parent.AddChild('{}');\n",
                        variable.name,
                    ))?;
                    file.write_fmt(format_args!(
                        "  {} := TimeOf(ISO8601ToDate(node.ChildNodes['{}'].Text));\n",
                        Self::first_char_uppercase(&variable.name),
                        variable.name
                    ))?;
                }
                DataType::Custom(name) => {
                    let type_alias = internal_representation
                        .types_aliases
                        .iter()
                        .find(|t| t.name == name.as_str());

                    match type_alias {
                        Some(t) => {
                            let mut pattern = t.pattern.clone();
                            let mut data_type = t.for_type.clone();

                            loop {
                                match &data_type {
                                    DataType::Custom(n) => {
                                        let type_alias = internal_representation
                                            .types_aliases
                                            .iter()
                                            .find(|t| t.name == n.as_str());

                                        if let Some(alias) = type_alias {
                                            if pattern.is_none() {
                                                pattern = alias.pattern.clone();
                                            }

                                            data_type = alias.for_type.clone();
                                        } else {
                                            break;
                                        }
                                    }
                                    _ => break,
                                }
                            }

                            match data_type {
                                DataType::Boolean => {
                                    file.write_fmt(format_args!(
                                        "  node := parent.AddChild('{}');\n",
                                        variable.name,
                                    ))?;
                                    file.write_fmt(format_args!(
                                        "  node.Text := IfThen({}, 'true', 'false');\n",
                                        Self::first_char_uppercase(&variable.name),
                                    ))?;
                                }
                                DataType::DateTime | DataType::Date if pattern.is_some() => {
                                    file.write_fmt(format_args!(
                                        "  node := parent.AddChild('{}');\n",
                                        variable.name,
                                    ))?;
                                    file.write_fmt(format_args!(
                                        "  node.Text := FormatDateTime('{}', {});\n",
                                        pattern.unwrap_or_default(),
                                        Self::first_char_uppercase(&variable.name),
                                    ))?;
                                }
                                DataType::DateTime | DataType::Date => {
                                    file.write_fmt(format_args!(
                                        "  node := parent.AddChild('{}');\n",
                                        variable.name,
                                    ))?;
                                    file.write_fmt(format_args!(
                                        "  node.Text := ISO8601ToDate({});\n",
                                        Self::first_char_uppercase(&variable.name),
                                    ))?;
                                }
                                DataType::Double => {
                                    file.write_fmt(format_args!(
                                        "  node := parent.AddChild('{}');\n",
                                        variable.name,
                                    ))?;
                                    file.write_fmt(format_args!(
                                        "  node.Text := FloatToStr({});\n",
                                        Self::first_char_uppercase(&variable.name),
                                    ))?;
                                }
                                DataType::Binary(BinaryEncoding::Base64) => {
                                    file.write_fmt(format_args!(
                                        "  node := parent.AddChild('{}');\n",
                                        variable.name,
                                    ))?;
                                    file.write_fmt(format_args!(
                                        "  node.Text := TNetEncoding.Base64.EncodeStringToBytes({});\n",
                                        Self::first_char_uppercase(&variable.name),
                                    ))?;
                                }
                                DataType::Binary(BinaryEncoding::Hex) => {
                                    file.write_fmt(format_args!(
                                        "  node := parent.AddChild('{}');\n",
                                        variable.name,
                                    ))?;
                                    file.write_fmt(format_args!(
                                        "  node.Text := BinToHexStr({});\n",
                                        Self::first_char_uppercase(&variable.name),
                                    ))?;
                                }
                                DataType::Integer => {
                                    file.write_fmt(format_args!(
                                        "  node := parent.AddChild('{}');\n",
                                        variable.name,
                                    ))?;
                                    file.write_fmt(format_args!(
                                        "  node.Text := IntToStr({});\n",
                                        Self::first_char_uppercase(&variable.name),
                                    ))?;
                                }
                                DataType::String => {
                                    file.write_fmt(format_args!(
                                        "  node := parent.AddChild('{}');\n",
                                        variable.name,
                                    ))?;
                                    file.write_fmt(format_args!(
                                        "  node.Text := {};\n",
                                        Self::first_char_uppercase(&variable.name),
                                    ))?;
                                }
                                DataType::Time if pattern.is_some() => {
                                    file.write_fmt(format_args!(
                                        "  node := parent.AddChild('{}');\n",
                                        variable.name,
                                    ))?;
                                    file.write_fmt(format_args!(
                                        "  node.Text := EncodeTime({}, '{}');\n",
                                        Self::first_char_uppercase(&variable.name),
                                        pattern.unwrap_or_default(),
                                    ))?;
                                }
                                DataType::Time => {
                                    file.write_fmt(format_args!(
                                        "  node := parent.AddChild('{}');\n",
                                        variable.name,
                                    ))?;
                                    file.write_fmt(format_args!(
                                        "  node.Text := TimeToStr({});\n",
                                        Self::first_char_uppercase(&variable.name),
                                    ))?;
                                }
                                _ => (),
                            }
                        }
                        None => {
                            file.write_fmt(format_args!(
                                "  node := parent.AddChild('{}');\n",
                                variable.name,
                            ))?;
                            file.write_fmt(format_args!(
                                "  {}.AppendToXmlRaw(node);\n",
                                Self::first_char_uppercase(&variable.name),
                            ))?;
                        }
                    }
                }
                DataType::List(_) => todo!(),
            }

            file.write_all(b"\n")?;
        }
        file.write(b"end;\n")?;

        file.write(b"\n")?;

        Ok(())
    }

    // Variable
    fn generate_variable_declaration(
        variable: &Variable,
        file: &mut File,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        file.write_fmt(format_args!(
            "{}{}: {};\n",
            " ".repeat(indentation),
            Self::first_char_uppercase(&variable.name),
            Self::get_datatype_language_representation(&variable.data_type),
        ))
    }

    // Helpers
    fn get_datatype_language_representation(datatype: &DataType) -> String {
        match datatype {
            DataType::Boolean => "Boolean".to_owned(),
            DataType::DateTime => "TDateTime".to_owned(),
            DataType::Date => "TDate".to_owned(),
            DataType::Double => "Double".to_owned(),
            DataType::Binary(_) => "TBytes".to_owned(),
            DataType::Integer => "TBytes".to_owned(),
            DataType::String => "String".to_owned(),
            DataType::Time => "TTime".to_owned(),
            DataType::Custom(c) => "T".to_owned() + Self::first_char_uppercase(c).as_str(),
            DataType::List(s) => {
                let gt = Self::get_datatype_language_representation(s);

                format!("TList<{}>", gt)
            }
        }
    }

    fn first_char_uppercase(name: &String) -> String {
        let mut graphemes = name.graphemes(true);

        match graphemes.next() {
            None => String::new(),
            Some(c) => c.to_uppercase() + graphemes.as_str(),
        }
    }

    fn first_char_lowercase(name: &String) -> String {
        let mut graphemes = name.graphemes(true);

        match graphemes.next() {
            None => String::new(),
            Some(c) => c.to_lowercase() + graphemes.as_str(),
        }
    }

    fn as_type_name(name: &String) -> String {
        String::from("T") + Self::first_char_uppercase(name).as_str()
    }
}

impl<'a> CodeGenerator<'a> for DelphiCodeGenerator<'a> {
    fn new(
        file: &'a mut File,
        unit_name: String,
        internal_representation: InternalRepresentation,
    ) -> Self {
        DelphiCodeGenerator {
            file,
            unit_name: unit_name.clone(),
            generate_date_time_helper: internal_representation.classes.iter().any(|c| {
                c.variables.iter().any(|v| match &v.data_type {
                    DataType::DateTime | DataType::Date | DataType::Time => true,
                    _ => false,
                })
            }) || internal_representation.types_aliases.iter().any(
                |a| match &a.for_type {
                    DataType::DateTime | DataType::Date | DataType::Time => true,
                    _ => false,
                },
            ),
            generate_hex_binary_helper: internal_representation.classes.iter().any(|c| {
                c.variables.iter().any(|v| match &v.data_type {
                    DataType::Binary(BinaryEncoding::Hex) => true,
                    _ => false,
                })
            }) || internal_representation.types_aliases.iter().any(
                |a| match &a.for_type {
                    DataType::Binary(BinaryEncoding::Hex) => true,
                    _ => false,
                },
            ),
            internal_representation,
        }
    }

    fn generate(&mut self) -> Result<(), std::io::Error> {
        self.write_unit()?;
        self.write_interface_start()?;
        self.write_uses()?;

        self.write_forward_declerations()?;
        self.write_declarations()?;

        self.write_implementation_start()?;
        self.write_implementation()?;
        // TODO: write implementation

        self.write_file_end()?;
        Ok(())
    }
}
