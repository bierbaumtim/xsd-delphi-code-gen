use std::collections::HashMap;
use std::fmt::Write;

use genphi_core::ir::{IrTypeIdOrName, types::*};

/// Represents a parsed code section with its content and manual additions
#[derive(Debug, Clone)]
pub struct CodeSection {
    pub marker_name: String,
    pub generated_content: String,
    pub manual_additions: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Configuration for code generation
#[derive(Debug, Clone)]
pub struct CodeGenConfig {
    pub generate_json_helpers: bool,
    pub generate_xml_helpers: bool,
    pub json_helper_unit: String,
    pub xml_helper_unit: String,
    pub date_format: String,
    pub indent_size: usize,
    pub preserve_existing_code: bool,
}

impl Default for CodeGenConfig {
    fn default() -> Self {
        Self {
            generate_json_helpers: true,
            generate_xml_helpers: true,
            json_helper_unit: "uJsonHelper".to_string(),
            xml_helper_unit: "uXmlHelper".to_string(),
            date_format: "yyyy-mm-dd".to_string(),
            indent_size: 2,
            preserve_existing_code: true,
        }
    }
}

/// Main code generator
pub struct DelphiCodeGenerator {
    config: CodeGenConfig,
    // existing_code_blocks: HashMap<String, String>,
    parsed_sections: HashMap<String, CodeSection>,
}

impl DelphiCodeGenerator {
    pub fn new(config: CodeGenConfig) -> Self {
        Self {
            config,
            // existing_code_blocks: HashMap::new(),
            parsed_sections: HashMap::new(),
        }
    }

    /// Parse existing code to identify marked sections and manual additions
    fn parse_marked_sections(&mut self, code: &str) {
        let lines: Vec<&str> = code.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Look for begin markers
            if line.starts_with("// __begin_") && line.ends_with("__") {
                let marker_name = line[11..line.len() - 2].to_string(); // Remove "// __begin_" and "__"
                let start_line = i;

                // Find corresponding end marker
                let end_marker = format!("// __end_{}__", marker_name);
                let mut end_line = i + 1;
                let mut generated_content = String::new();
                let mut in_generated_block = true;
                let mut manual_additions = String::new();

                while end_line < lines.len() {
                    let current_line = lines[end_line];

                    if current_line.trim() == end_marker {
                        break;
                    }

                    // Check if we hit a nested begin marker (generated content)
                    if in_generated_block && current_line.trim().starts_with("// __begin_") {
                        generated_content.push_str(current_line);
                        generated_content.push('\n');
                    }
                    // Check if we hit a nested end marker (end of generated content)
                    else if in_generated_block && current_line.trim().starts_with("// __end_") {
                        generated_content.push_str(current_line);
                        generated_content.push('\n');
                        in_generated_block = false; // Switch to manual additions
                    }
                    // Regular generated content
                    else if in_generated_block {
                        generated_content.push_str(current_line);
                        generated_content.push('\n');
                    }
                    // Manual additions (after generated content)
                    else {
                        manual_additions.push_str(current_line);
                        manual_additions.push('\n');
                    }

                    end_line += 1;
                }

                if end_line < lines.len() {
                    self.parsed_sections.insert(
                        marker_name.clone(),
                        CodeSection {
                            marker_name,
                            generated_content: generated_content.trim_end().to_string(),
                            manual_additions: manual_additions.trim_end().to_string(),
                            start_line,
                            end_line,
                        },
                    );
                }

                i = end_line;
            }

            i += 1;
        }
    }

    /// Generate comment marker for the beginning of a section
    fn begin_marker(&self, section_name: &str) -> String {
        format!("  // __begin_{}__", section_name)
    }

    /// Generate comment marker for the end of a section
    fn end_marker(&self, section_name: &str) -> String {
        format!("  // __end_{}__", section_name)
    }

    /// Get preserved manual additions for a section
    fn get_manual_additions(&self, section_name: &str) -> String {
        if let Some(section) = self.parsed_sections.get(section_name) {
            if !section.manual_additions.is_empty() {
                format!("\n{}", section.manual_additions)
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }

    /// Generate unit header with manual edit instructions
    fn generate_unit_header(&self, unit_name: &str) -> String {
        format!(
            r#"unit u{}Models;

{{
  Auto-generated Delphi unit with manual edit support

  HOW TO ADD MANUAL EDITS:

  1. Manual code can be added AFTER each generated block
  2. Generated blocks are marked with comments: // __begin_section__ ... // __end_section__
  3. Add your custom code AFTER the // __end_section__ marker
  4. Supported manual additions:
     - Private fields (after // __end_fields__)
     - Private/Public methods (after // __end_methods__)
     - Properties (after // __end_properties__)
     - Custom implementations (after // __end_class_implementation__)

  EXAMPLE:
    // __begin_fields__
    FGeneratedField: String;
    // __end_fields__
    FMyCustomField: Integer;  // <- Manual addition here

  WARNING: Do not modify code INSIDE the marked blocks - it will be overwritten!
}}

interface"#,
            unit_name
        )
    }

    /// Parse existing unit to preserve custom code blocks
    pub fn parse_existing_unit(&mut self, existing_code: &str) {
        if !self.config.preserve_existing_code {
            return;
        }

        // // Extract custom code blocks that should be preserved
        // self.extract_custom_code_blocks(existing_code);

        // Parse marked sections for precise updates
        self.parse_marked_sections(existing_code);
    }

    /// Generate complete Delphi unit
    pub fn generate_unit(&self, unit: &DelphiUnit) -> anyhow::Result<String> {
        let mut output = String::new();

        // Unit header with manual edit instructions
        output.push_str(&self.generate_unit_header(&unit.unit_name));
        writeln!(output)?;
        writeln!(output)?;

        let mut interface_uses = unit.uses_interface.clone();
        if unit.classes.iter().any(|c| c.generate_json_support) {
            interface_uses.push("System.JSON".to_owned());
        }

        // Uses clause (interface)
        if !interface_uses.is_empty() {
            writeln!(output, "uses")?;
            for (i, use_unit) in interface_uses.iter().enumerate() {
                if i == interface_uses.len() - 1 {
                    writeln!(output, "  {};", use_unit)?;
                } else {
                    writeln!(output, "  {},", use_unit)?;
                }
            }
            writeln!(output)?;
        }

        writeln!(output, "type")?;

        // Forward declarations
        if !unit.classes.is_empty() {
            writeln!(output, "  {{$REGION 'Forward Declarations'}}")?;
            writeln!(output, "{}", self.begin_marker("forward_declarations"))?;
            for decl in &unit.classes {
                writeln!(output, "  {};", decl.name)?;
            }
            writeln!(output, "{}", self.end_marker("forward_declarations"))?;
            output.push_str(&self.get_manual_additions("forward_declarations"));
            writeln!(output, "  {{$ENDREGION}}")?;
            writeln!(output)?;
        }

        // Enums and helpers
        if !unit.enums.is_empty() {
            writeln!(output, "  {{$REGION 'Enums and Helpers'}}")?;
            writeln!(output, "  {{$SCOPEDENUMS ON}}")?;
            writeln!(output, "{}", self.begin_marker("enums"))?;

            for enum_def in &unit.enums {
                self.generate_enum_declaration(&mut output, enum_def)?;
                self.generate_enum_helper_declaration(&mut output, enum_def)?;
            }

            writeln!(output, "{}", self.end_marker("enums"))?;
            output.push_str(&self.get_manual_additions("enums"));
            writeln!(output, "  {{$SCOPEDENUMS OFF}}")?;
            writeln!(output, "  {{$ENDREGION}}")?;
            writeln!(output)?;
        }

        // Records
        if !unit.records.is_empty() {
            writeln!(output, "  {{$REGION 'Records'}}")?;
            writeln!(output, "{}", self.begin_marker("records"))?;
            for record in &unit.records {
                self.generate_record_declaration(&mut output, record)?;
            }
            writeln!(output, "{}", self.end_marker("records"))?;
            output.push_str(&self.get_manual_additions("records"));
            writeln!(output, "  {{$ENDREGION}}")?;
            writeln!(output)?;
        }

        // Classes
        if !unit.classes.is_empty() {
            writeln!(output, "  {{$REGION 'Models'}}")?;
            writeln!(output, "{}", self.begin_marker("classes"))?;
            for class in &unit.classes {
                self.generate_class_declaration(&mut output, class)?;
            }
            writeln!(output, "{}", self.end_marker("classes"))?;
            output.push_str(&self.get_manual_additions("classes"));
            writeln!(output, "  {{$ENDREGION}}")?;
            writeln!(output)?;
        }

        // Implementation section
        writeln!(output, "implementation")?;
        writeln!(output)?;

        // Uses clause (implementation)
        let mut impl_uses = unit.uses_implementation.clone();
        if self.config.generate_json_helpers {
            impl_uses.push(self.config.json_helper_unit.clone());
        }
        if self.config.generate_xml_helpers {
            impl_uses.push(self.config.xml_helper_unit.clone());
        }
        impl_uses.extend_from_slice(&[
            "System.DateUtils".to_string(),
            "System.SysUtils".to_string(),
        ]);

        if !impl_uses.is_empty() {
            writeln!(output, "uses")?;
            for (i, use_unit) in impl_uses.iter().enumerate() {
                if i == impl_uses.len() - 1 {
                    writeln!(output, "  {};", use_unit)?;
                } else {
                    writeln!(output, "  {},", use_unit)?;
                }
            }
            writeln!(output)?;
        }

        // Constants
        // if !unit.constants.is_empty() {
        //     for (name, value) in &unit.constants {
        //         writeln!(output, "const")?;
        //         writeln!(output, "  {}: string = '{}';", name, value)?;
        //     }
        //     writeln!(output)?;
        // }

        // Enum helpers implementation
        if !unit.enums.is_empty() {
            writeln!(output, "{{$REGION 'Enum Helpers'}}")?;
            writeln!(output, "{}", self.begin_marker("enum_helpers_impl"))?;
            for enum_def in &unit.enums {
                self.generate_enum_helper_implementation(&mut output, enum_def)?;
            }
            writeln!(output, "{}", self.end_marker("enum_helpers_impl"))?;
            output.push_str(&self.get_manual_additions("enum_helpers_impl"));
            writeln!(output, "{{$ENDREGION}}")?;
            writeln!(output)?;
        }

        // Record implementations
        if !unit.records.is_empty() {
            writeln!(output, "{{$REGION 'Records'}}")?;
            writeln!(output, "{}", self.begin_marker("records_impl"))?;
            for record in &unit.records {
                self.generate_record_implementation(&mut output, record)?;
            }
            writeln!(output, "{}", self.end_marker("records_impl"))?;
            output.push_str(&self.get_manual_additions("records_impl"));
            writeln!(output, "{{$ENDREGION}}")?;
            writeln!(output)?;
        }

        // Class implementations
        if !unit.classes.is_empty() {
            writeln!(output, "{{$REGION 'Models'}}")?;
            writeln!(output, "{}", self.begin_marker("class_implementations"))?;
            for class in &unit.classes {
                self.generate_class_implementation(&mut output, class)?;
            }
            writeln!(output, "{}", self.end_marker("class_implementations"))?;
            output.push_str(&self.get_manual_additions("class_implementations"));
            writeln!(output, "{{$ENDREGION}}")?;
            writeln!(output)?;
        }

        writeln!(output, "end.")?;

        Ok(output)
    }

    fn generate_enum_declaration(
        &self,
        output: &mut String,
        enum_def: &DelphiEnum,
    ) -> anyhow::Result<()> {
        if let Some(comment) = &enum_def.comment {
            writeln!(output, "  // {}", comment)?;
        }

        write!(output, "  {} = (", enum_def.name)?;

        for (i, variant) in enum_def.variants.iter().enumerate() {
            if i > 0 {
                write!(output, ", ")?;
            }

            if let Some(value) = &variant.value {
                write!(output, "{} = {}", variant.name, value)?;
            } else {
                write!(output, "{}", variant.name)?;
            }
        }

        writeln!(output, ");")?;
        writeln!(output)?;

        Ok(())
    }

    fn generate_enum_helper_declaration(
        &self,
        output: &mut String,
        enum_def: &DelphiEnum,
    ) -> anyhow::Result<()> {
        writeln!(
            output,
            "  {}Helper = record helper for {}",
            enum_def.name, enum_def.name
        )?;
        writeln!(
            output,
            "    class function FromString(const pValue: String): {}; static;",
            enum_def.name
        )?;
        writeln!(output, "    function ToString: String;")?;
        writeln!(output, "  end;")?;
        writeln!(output)?;

        Ok(())
    }

    fn generate_enum_helper_implementation(
        &self,
        output: &mut String,
        enum_def: &DelphiEnum,
    ) -> anyhow::Result<()> {
        writeln!(output, "{{ {}Helper }}", enum_def.name)?;

        // FromString method
        writeln!(
            output,
            "class function {}Helper.FromString(const pValue: String): {};",
            enum_def.name, enum_def.name
        )?;
        writeln!(output, "begin")?;

        for (i, variant) in enum_def.variants.iter().enumerate() {
            let condition = if i == 0 { "if" } else { "else if" };
            writeln!(
                output,
                "  {} LowerCase(pValue) = '{}' then",
                condition,
                variant.name.to_lowercase()
            )?;
            writeln!(output, "    Result := {}.{}", enum_def.name, variant.name)?;
        }

        writeln!(output, "  else")?;
        writeln!(
            output,
            "    raise Exception.CreateFmt('Unknown {} value: %s', [pValue]);",
            enum_def.name
        )?;
        writeln!(output, "end;")?;
        writeln!(output)?;

        // ToString method
        writeln!(output, "function {}Helper.ToString: String;", enum_def.name)?;
        writeln!(output, "begin")?;
        writeln!(output, "  case Self of")?;

        for variant in &enum_def.variants {
            writeln!(
                output,
                "    {}.{}: Result := '{}';",
                enum_def.name,
                variant.name,
                variant.name.to_lowercase()
            )?;
        }

        writeln!(output, "  else")?;
        writeln!(output, "    Result := 'Unknown';")?;
        writeln!(output, "  end;")?;
        writeln!(output, "end;")?;
        writeln!(output)?;

        Ok(())
    }

    fn generate_record_declaration(
        &self,
        output: &mut String,
        record: &DelphiRecord,
    ) -> anyhow::Result<()> {
        if let Some(comment) = &record.comment {
            writeln!(output, "  // {}", comment)?;
        }

        writeln!(output, "  {} = record", record.name)?;

        // Fields
        if !record.fields.is_empty() {
            for field in &record.fields {
                self.generate_field_declaration(output, field, "    ")?;
            }
            writeln!(output)?;
        }

        // Methods
        if !record.methods.is_empty() {
            for method in &record.methods {
                self.generate_method_declaration(output, method, "    ")?;
            }
        }

        writeln!(output, "  end;")?;
        writeln!(output)?;

        Ok(())
    }

    fn generate_class_declaration(
        &self,
        output: &mut String,
        class: &DelphiClass,
    ) -> anyhow::Result<()> {
        if let Some(comment) = &class.comment {
            writeln!(output, "  // {}", comment)?;
        }

        if let Some(parent) = &class.parent_class {
            writeln!(output, "  {} = class({})", class.name, parent)?;
        } else {
            writeln!(output, "  {} = class", class.name)?;
        }

        // Group fields and methods by visibility
        let mut visibility_groups: HashMap<
            DelphiVisibility,
            (Vec<&DelphiField>, Vec<&DelphiMethod>),
        > = HashMap::new();

        for field in &class.fields {
            visibility_groups
                .entry(field.visibility.clone())
                .or_default()
                .0
                .push(field);
        }

        for method in &class.methods {
            visibility_groups
                .entry(method.visibility.clone())
                .or_default()
                .1
                .push(method);
        }

        // Generate in visibility order
        let visibility_order = [
            DelphiVisibility::StrictPrivate,
            DelphiVisibility::Private,
            DelphiVisibility::Protected,
            DelphiVisibility::Public,
            DelphiVisibility::Published,
        ];

        for visibility in visibility_order {
            if let Some((fields, methods)) = visibility_groups.get(&visibility) {
                if !fields.is_empty()
                    || !methods.is_empty()
                    || (visibility == DelphiVisibility::Public && class.generate_json_support)
                {
                    writeln!(output, "  {}", self.visibility_to_string(&visibility))?;

                    // Generate fields with markers
                    if !fields.is_empty() {
                        let fields_marker = format!("fields_{}", class.name.to_lowercase());
                        writeln!(output, "{}", self.begin_marker(&fields_marker))?;
                        for field in fields {
                            self.generate_field_declaration(output, field, "    ")?;
                        }
                        writeln!(output, "{}", self.end_marker(&fields_marker))?;
                        output.push_str(&self.get_manual_additions(&fields_marker));
                    }

                    if !fields.is_empty()
                        && (!methods.is_empty()
                            || (visibility == DelphiVisibility::Public
                                && class.generate_json_support))
                    {
                        writeln!(output)?;
                    }

                    if visibility == DelphiVisibility::Public && class.generate_json_support {
                        let json_marker = format!("json_support_{}", class.name.to_lowercase());
                        writeln!(output, "{}", self.begin_marker(&json_marker))?;
                        writeln!(output, "    constructor FromJson(const pJson: string);")?;
                        writeln!(
                            output,
                            "    constructor FromJsonRaw(const pJson: TJSONValue);"
                        )?;
                        writeln!(output, "{}", self.end_marker(&json_marker))?;
                        output.push_str(&self.get_manual_additions(&json_marker));

                        if !methods.is_empty() {
                            writeln!(output)?;
                        }
                    }

                    // Generate methods with markers
                    if !methods.is_empty() {
                        let methods_marker = format!(
                            "methods_{}_{}",
                            self.visibility_to_string(&visibility).replace(" ", "_"),
                            class.name.to_lowercase()
                        );
                        writeln!(output, "{}", self.begin_marker(&methods_marker))?;
                        for method in methods {
                            self.generate_method_declaration(output, method, "    ")?;
                        }
                        writeln!(output, "{}", self.end_marker(&methods_marker))?;
                        output.push_str(&self.get_manual_additions(&methods_marker));
                    }
                }
            } else if visibility == DelphiVisibility::Public && class.generate_json_support {
                writeln!(output, "  {}", self.visibility_to_string(&visibility))?;

                let json_marker = format!("json_support_{}", class.name.to_lowercase());
                writeln!(output, "{}", self.begin_marker(&json_marker))?;
                writeln!(output, "    constructor FromJson(const pJson: string);")?;
                writeln!(
                    output,
                    "    constructor FromJsonRaw(const pJson: TJSONValue);"
                )?;
                writeln!(output, "{}", self.end_marker(&json_marker))?;
                output.push_str(&self.get_manual_additions(&json_marker));
            }
        }

        // Properties with markers
        if !class.properties.is_empty() {
            writeln!(output, "  public")?;
            let properties_marker = format!("properties_{}", class.name.to_lowercase());
            writeln!(output, "{}", self.begin_marker(&properties_marker))?;
            for property in &class.properties {
                self.generate_property_declaration(output, property, "    ")?;
            }
            writeln!(output, "{}", self.end_marker(&properties_marker))?;
            output.push_str(&self.get_manual_additions(&properties_marker));
        }

        writeln!(output, "  end;")?;
        writeln!(output)?;

        Ok(())
    }

    fn generate_field_declaration(
        &self,
        output: &mut String,
        field: &DelphiField,
        indent: &str,
    ) -> anyhow::Result<()> {
        if let Some(comment) = &field.comment {
            writeln!(output, "{}// {}", indent, comment)?;
        }
        writeln!(
            output,
            "{indent}F{}: {};",
            field.name,
            field.field_type.as_type_name()
        )?;

        Ok(())
    }

    fn generate_method_declaration(
        &self,
        output: &mut String,
        method: &DelphiMethod,
        indent: &str,
    ) -> anyhow::Result<()> {
        if let Some(comment) = &method.comment {
            writeln!(output, "{}// {}", indent, comment)?;
        }

        let mut method_line = String::new();

        if method.is_class_method {
            method_line.push_str("class ");
        }

        if method.is_constructor {
            method_line.push_str("constructor ");
        } else if method.return_type.is_some() {
            method_line.push_str("function ");
        } else {
            method_line.push_str("procedure ");
        }

        method_line.push_str(&method.name);

        if !method.parameters.is_empty() {
            method_line.push('(');
            for (i, param) in method.parameters.iter().enumerate() {
                if i > 0 {
                    method_line.push_str("; ");
                }

                if param.is_const {
                    method_line.push_str("const ");
                } else if param.is_var {
                    method_line.push_str("var ");
                } else if param.is_out {
                    method_line.push_str("out ");
                }

                method_line.push_str(&format!(
                    "{}: {}",
                    param.name,
                    param.param_type.as_type_name()
                ));

                if let Some(default) = &param.default_value {
                    method_line.push_str(&format!(" = {}", default));
                }
            }
            method_line.push(')');
        }

        if let Some(return_type) = &method.return_type {
            method_line.push_str(&format!(": {}", return_type));
        }

        method_line.push(';');

        if method.is_virtual {
            method_line.push_str(" virtual;");
        } else if method.is_override {
            method_line.push_str(" override;");
        }

        if method.is_static {
            method_line.push_str(" static;");
        }

        writeln!(output, "{}{}", indent, method_line)?;

        Ok(())
    }

    fn generate_property_declaration(
        &self,
        output: &mut String,
        property: &DelphiProperty,
        indent: &str,
    ) -> anyhow::Result<()> {
        if let Some(comment) = &property.comment {
            writeln!(output, "{}// {}", indent, comment)?;
        }

        let mut prop_line = format!(
            "{}property {}: {}",
            indent,
            property.name,
            property.property_type.as_type_name(),
        );

        if let Some(getter) = &property.getter {
            prop_line.push_str(&format!(" read {}", getter));
        }

        if let Some(setter) = &property.setter {
            prop_line.push_str(&format!(" write {}", setter));
        }

        prop_line.push(';');

        writeln!(output, "{}", prop_line)?;

        Ok(())
    }

    fn generate_record_implementation(
        &self,
        output: &mut String,
        record: &DelphiRecord,
    ) -> anyhow::Result<()> {
        // Generate method implementations
        for method in &record.methods {
            self.generate_method_implementation(
                output,
                method,
                &record.name,
                record.generate_json_support,
                record.generate_xml_support,
            )?;
        }

        Ok(())
    }

    fn generate_class_implementation(
        &self,
        output: &mut String,
        class: &DelphiClass,
    ) -> anyhow::Result<()> {
        let class_impl_marker = format!("class_impl_{}", class.name.to_lowercase());
        writeln!(output, "{}", self.begin_marker(&class_impl_marker))?;

        // Generate constants for JSON keys
        if class.generate_json_support {
            writeln!(output, "const")?;

            for field in &class.fields {
                if let Some(json_key) = &field.json_key {
                    writeln!(
                        output,
                        "  cn{}JsonKey: string = '{}';",
                        field.name, json_key
                    )?;
                }
            }

            writeln!(output)?;
        }

        // Generate method implementations
        for method in &class.methods {
            self.generate_method_implementation(
                output,
                method,
                &class.name,
                class.generate_json_support,
                class.generate_xml_support,
            )?;
        }

        // Auto-generate JSON constructors if needed
        if class.generate_json_support {
            self.generate_auto_json_constructor(output, class)?;
        }

        // Auto-generate destructor if needed
        let has_reference_types = class.fields.iter().any(|f| f.is_reference_type);
        if has_reference_types {
            self.generate_auto_destructor(output, class)?;
        }

        writeln!(output, "{}", self.end_marker(&class_impl_marker))?;
        output.push_str(&self.get_manual_additions(&class_impl_marker));

        Ok(())
    }

    fn generate_method_implementation(
        &self,
        output: &mut String,
        method: &DelphiMethod,
        type_name: &str,
        json_support: bool,
        _xml_support: bool,
    ) -> anyhow::Result<()> {
        // Check if we have existing implementation to preserve
        // let method_key = format!("{}.{}", type_name, method.name);
        // if let Some(existing_impl) = self.existing_code_blocks.get(&method_key) {
        //     writeln!(output, "{}", existing_impl)?;
        //     writeln!(output)?;
        //     return Ok(());
        // }

        writeln!(output, "{{ {} }}", type_name)?;

        let mut method_signature = String::new();

        if method.is_class_method {
            method_signature.push_str("class ");
        }

        if method.is_constructor {
            method_signature.push_str("constructor ");
        } else if method.return_type.is_some() {
            method_signature.push_str("function ");
        } else {
            method_signature.push_str("procedure ");
        }

        method_signature.push_str(&format!("{}.{}", type_name, method.name));

        if !method.parameters.is_empty() {
            method_signature.push('(');
            for (i, param) in method.parameters.iter().enumerate() {
                if i > 0 {
                    method_signature.push_str("; ");
                }

                if param.is_const {
                    method_signature.push_str("const ");
                } else if param.is_var {
                    method_signature.push_str("var ");
                } else if param.is_out {
                    method_signature.push_str("out ");
                }

                method_signature.push_str(&format!(
                    "{}: {}",
                    param.name,
                    param.param_type.as_type_name()
                ));
            }
            method_signature.push(')');
        }

        if let Some(return_type) = &method.return_type {
            method_signature.push_str(&format!(": {}", return_type));
        }

        method_signature.push(';');

        writeln!(output, "{}", method_signature)?;
        writeln!(output, "begin")?;

        // Generate basic implementation based on method type
        if method.is_constructor && method.name == "FromJson" && json_support {
            self.generate_json_constructor_body(output, method)?;
        } else if method.is_constructor && method.name == "FromJsonRaw" && json_support {
            self.generate_json_raw_constructor_body(output, method)?;
        } else {
            writeln!(output, "  // TODO: Implement {}", method.name)?;
            if method.return_type.is_some() {
                writeln!(
                    output,
                    "  Result := Default({});",
                    method.return_type.as_ref().unwrap()
                )?;
            }
        }

        writeln!(output, "end;")?;
        writeln!(output)?;

        Ok(())
    }

    fn generate_json_constructor_body(
        &self,
        output: &mut String,
        _method: &DelphiMethod,
    ) -> anyhow::Result<()> {
        writeln!(output, "  var vRoot := TJSONObject.ParseJSONValue(pJson);")?;
        writeln!(output)?;
        writeln!(output, "  try")?;
        writeln!(output, "    FromJsonRaw(vRoot);")?;
        writeln!(output, "  finally")?;
        writeln!(output, "    FreeAndNil(vRoot);")?;
        writeln!(output, "  end;")?;

        Ok(())
    }

    fn generate_json_raw_constructor_body(
        &self,
        output: &mut String,
        _method: &DelphiMethod,
    ) -> anyhow::Result<()> {
        writeln!(output, "  inherited Create;")?;
        writeln!(output)?;
        writeln!(output, "  // TODO: Parse JSON fields")?;

        Ok(())
    }

    fn generate_auto_json_constructor(
        &self,
        output: &mut String,
        class: &DelphiClass,
    ) -> anyhow::Result<()> {
        // FromJson constructor
        writeln!(
            output,
            "constructor {}.FromJson(const pJson: String);",
            class.name
        )?;
        writeln!(output, "begin")?;
        writeln!(output, "  var vRoot := TJSONObject.ParseJSONValue(pJson);")?;
        writeln!(output)?;
        writeln!(output, "  try")?;
        writeln!(output, "    FromJsonRaw(vRoot);")?;
        writeln!(output, "  finally")?;
        writeln!(output, "    FreeAndNil(vRoot);")?;
        writeln!(output, "  end;")?;
        writeln!(output, "end;")?;
        writeln!(output)?;

        // FromJsonRaw constructor
        writeln!(
            output,
            "constructor {}.FromJsonRaw(pJson: TJSONValue);",
            class.name
        )?;
        writeln!(output, "begin")?;
        writeln!(output, "  inherited Create;")?;
        writeln!(output)?;

        for field in &class.fields {
            if let Some(_json_key) = &field.json_key {
                let const_name = format!("cn{}JsonKey", field.name);
                let default_value = field
                    .default_value
                    .as_ref()
                    .map(|v| v.as_str())
                    .unwrap_or("");

                if let DelphiType::Class(IrTypeIdOrName::Name(name)) = &field.field_type {
                    if field.is_required {
                        writeln!(
                            output,
                            "  F{} := {name}.FromJsonRaw(pJson.GetValue<TJSONObject>({const_name}));",
                            field.name
                        )?;
                    } else {
                        writeln!(output, "  var __{}Obj: TJSONObject;", field.name)?;
                        writeln!(
                            output,
                            "  if pJson.TryGetValue<TJSONObject>({const_name}, __{}Obj) then begin",
                            field.name
                        )?;
                        writeln!(
                            output,
                            "    F{} := {name}.FromJsonRaw(__{}Obj);",
                            field.name, field.name
                        )?;
                        writeln!(output, "  end else begin")?;
                        writeln!(output, "  F{} := nil;", field.name)?;
                        writeln!(output, "  end;")?;
                    }
                } else if let DelphiType::Enum(IrTypeIdOrName::Name(name)) = &field.field_type {
                    writeln!(
                        output,
                        "  F{} := {name}.FromString(TJsonHelper.TryGetValueOrDefault<TJSONString, String>(pJson, {const_name}, '{default_value}'));",
                        field.name
                    )?;
                } else if let DelphiType::List(inner_type) = &field.field_type {
                    let is_object_list = matches!(
                        inner_type.as_ref(),
                        DelphiType::Class(_) | DelphiType::List(_)
                    );
                    let helper_function = match (is_object_list, field.is_required) {
                        (true, true) => "TJSONHelper.DeserializeObjectList",
                        (true, false) => "TJSONHelper.DeserializeOptionalObjectList",
                        (false, true) => "TJSONHelper.DeserializeList",
                        (false, false) => "TJSONHelper.DeserializeOptionalList",
                    };

                    writeln!(
                        output,
                        "  F{} := {helper_function}<{}>(",
                        field.name,
                        inner_type.as_ref().as_type_name()
                    )?;
                    writeln!(output, "    pJson,")?;
                    writeln!(output, "    {const_name},")?;
                    writeln!(
                        output,
                        "    function (pJson: TJSONValue): {}",
                        inner_type.as_ref().as_type_name()
                    )?;
                    writeln!(output, "    begin")?;
                    match inner_type.as_ref() {
                        DelphiType::List(_) => writeln!(
                            output,
                            "      // TODO: Implement nested list conversion logic"
                        )?,
                        DelphiType::Enum(IrTypeIdOrName::Name(name)) => writeln!(
                            output,
                            "      Result := {name}.FromString(TJSONString(pJson).Value);"
                        )?,
                        DelphiType::Class(IrTypeIdOrName::Name(name)) => {
                            writeln!(output, "      Result := {name}.FromJsonRaw(pJson);")?
                        }
                        DelphiType::Binary => {
                            writeln!(output, "      Result := TJSONBool(pJson).AsBoolean;")?
                        }
                        DelphiType::Boolean => {
                            writeln!(output, "      Result := TJSONBool(pJson).AsBoolean;")?
                        }
                        DelphiType::DateTime => writeln!(
                            output,
                            "      Result := ISO8601ToDate(TJSONString(pJson).Value);"
                        )?,
                        DelphiType::Double | DelphiType::Float => {
                            writeln!(output, "      Result := TJSONNumber(pJson).AsDouble;")?
                        }
                        DelphiType::Integer => {
                            writeln!(output, "      Result := TJSONNumber(pJson).AsInt;")?
                        }
                        DelphiType::Pointer => writeln!(output, "      Result := nil;")?,
                        DelphiType::String => {
                            writeln!(output, "      Result := TJSONString(pJson).Value;")?
                        }
                        _ => writeln!(output, "      // TODO: Implement conversion logic")?,
                    }
                    writeln!(output, "    end")?;
                    writeln!(output, "  );")?;
                } else if field.field_type == DelphiType::Binary {
                    writeln!(
                        output,
                        "  F{} := TNetEncoding.Base64.DecodeStringToBytes(TJsonHelper.TryGetValueOrDefault<TJSONString, String>(pJson, {const_name}, '{default_value}'));",
                        field.name
                    )?;
                } else if field.field_type == DelphiType::DateTime {
                    writeln!(
                        output,
                        "  var __{} := TJsonHelper.TryGetValueOrDefault<TJSONString, String>(pJson, {const_name}, '{default_value}');",
                        field.name
                    )?;
                    writeln!(output, "  if __{} <> '' then begin", field.name)?;
                    writeln!(
                        output,
                        "    F{} := ISO8601ToDate(__{});",
                        field.name, field.name
                    )?;
                    writeln!(output, "  end else begin")?;
                    writeln!(output, "    F{} := 0;", field.name,)?;
                    writeln!(output, "  end;")?;
                } else if field.field_type == DelphiType::Pointer {
                    writeln!(output, "  F{} := nil;", field.name)?;
                } else {
                    let (json_type, data_type, default_value) = match &field.field_type {
                        DelphiType::Boolean => ("TJSONBool", "Boolean", "False"),
                        DelphiType::Double => ("TJSONNumber", "Double", "0.0"),
                        DelphiType::Float => ("TJSONNumber", "Float", "0.0"),
                        DelphiType::Integer => ("TJSONNumber", "Integer", "0"),
                        DelphiType::String => ("TJSONString", "String", "''"),
                        _ => continue,
                    };

                    writeln!(
                        output,
                        "  F{} := TJsonHelper.TryGetValueOrDefault<{json_type}, {data_type}>(pJson, {const_name}, {default_value});",
                        field.name
                    )?;
                }
            }
        }

        writeln!(output, "end;")?;
        writeln!(output)?;

        Ok(())
    }

    fn generate_auto_destructor(
        &self,
        output: &mut String,
        class: &DelphiClass,
    ) -> anyhow::Result<()> {
        writeln!(output, "destructor {}.Destroy;", class.name)?;
        writeln!(output, "begin")?;

        for field in &class.fields {
            if field.is_reference_type {
                writeln!(output, "  FreeAndNil(F{});", field.name)?;
            }
        }

        writeln!(output)?;
        writeln!(output, "  inherited;")?;
        writeln!(output, "end;")?;
        writeln!(output)?;

        Ok(())
    }

    fn visibility_to_string(&self, visibility: &DelphiVisibility) -> &'static str {
        match visibility {
            DelphiVisibility::Private => "private",
            DelphiVisibility::StrictPrivate => "strict private",
            DelphiVisibility::Protected => "protected",
            DelphiVisibility::Public => "public",
            DelphiVisibility::Published => "published",
        }
    }

    /// Update existing unit with minimal changes while preserving manual edits
    pub fn update_unit(
        &mut self,
        existing_code: &str,
        new_unit: &DelphiUnit,
    ) -> anyhow::Result<String> {
        // Parse the existing code to extract manual additions
        self.parse_existing_unit(existing_code);

        // Generate the new unit with preserved manual additions
        self.generate_unit(new_unit)
    }

    /// Update specific sections of existing code
    pub fn update_sections(
        &mut self,
        existing_code: &str,
        sections_to_update: Vec<String>,
        new_unit: &DelphiUnit,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Parse existing code
        self.parse_existing_unit(existing_code);

        // Generate new code for specific sections only
        let new_full_code = self.generate_unit(new_unit)?;

        // Replace only the specified sections in the existing code
        let mut result = existing_code.to_string();

        for section_name in sections_to_update {
            if let Some(new_section) =
                self.extract_section_from_generated(&new_full_code, &section_name)
            {
                result = self.replace_section_in_code(&result, &section_name, &new_section)?;
            }
        }

        Ok(result)
    }

    /// Extract a specific section from generated code
    fn extract_section_from_generated(
        &self,
        generated_code: &str,
        section_name: &str,
    ) -> Option<String> {
        let begin_marker = format!("// __begin_{}__", section_name);
        let end_marker = format!("// __end_{}__", section_name);

        if let Some(start_pos) = generated_code.find(&begin_marker) {
            if let Some(end_pos) = generated_code.find(&end_marker) {
                if end_pos > start_pos {
                    let section = &generated_code[start_pos..end_pos + end_marker.len()];
                    return Some(section.to_string());
                }
            }
        }

        None
    }

    /// Replace a specific section in existing code
    fn replace_section_in_code(
        &self,
        existing_code: &str,
        section_name: &str,
        new_section: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let begin_marker = format!("// __begin_{}__", section_name);
        let end_marker = format!("// __end_{}__", section_name);

        if let Some(start_pos) = existing_code.find(&begin_marker) {
            // Find the end of the section (including manual additions)
            let end_search_start = start_pos + begin_marker.len();

            // Look for the end marker
            if let Some(end_marker_pos) = existing_code[end_search_start..].find(&end_marker) {
                let absolute_end_pos = end_search_start + end_marker_pos + end_marker.len();

                // Look for the next begin marker to find where manual additions end
                let manual_additions_end = if let Some(next_begin) =
                    existing_code[absolute_end_pos..].find("// __begin_")
                {
                    absolute_end_pos + next_begin
                } else {
                    // If no next section, look for other structural markers
                    let markers = ["{$ENDREGION}", "implementation", "end."];
                    let mut next_structural = existing_code.len();

                    for marker in &markers {
                        if let Some(pos) = existing_code[absolute_end_pos..].find(marker) {
                            let absolute_pos = absolute_end_pos + pos;
                            if absolute_pos < next_structural {
                                next_structural = absolute_pos;
                            }
                        }
                    }
                    next_structural
                };

                // Extract manual additions
                let manual_additions = existing_code[absolute_end_pos..manual_additions_end].trim();

                // Reconstruct the code
                let mut result = String::new();
                result.push_str(&existing_code[..start_pos]);
                result.push_str(new_section);
                if !manual_additions.is_empty() {
                    result.push('\n');
                    result.push_str(manual_additions);
                }
                result.push_str(&existing_code[manual_additions_end..]);

                return Ok(result);
            }
        }

        // If we can't find the section, return the original code
        Ok(existing_code.to_string())
    }
}
