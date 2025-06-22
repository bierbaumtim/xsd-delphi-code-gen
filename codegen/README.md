# Delphi Code Generator

A comprehensive Rust library for generating Delphi/Pascal code with advanced update capabilities and intelligent preservation of manual edits.

## Features

- **Type-safe code generation** for Delphi units
- **Enum generation** with helper classes for string conversion
- **Record types** with methods and serialization support
- **Class generation** with properties, methods, and inheritance
- **JSON serialization/deserialization** support
- **XML serialization/deserialization** support
- **🔄 Intelligent update system** - preserves manual edits during regeneration
- **📍 Comment-based markers** - precise section identification and updates
- **🎯 Selective updates** - update only specific sections when needed
- **📖 Auto-generated documentation** - explains how to add manual edits
- **Builder patterns** for easy construction of Delphi types
- **Configurable output** with customizable formatting and features

## Quick Start

### Basic Usage

```rust
use delphi_code_gen::*;

// Create an enum with helper
let status_enum = DelphiEnumBuilder::new("TStatus")
    .add_variant("None")
    .add_variant("Active")
    .add_variant("Inactive")
    .comment("Entity status enumeration")
    .build();

// Create a class with JSON support
let user_class = DelphiClassBuilder::new("TUser")
    .add_string_field("Id", Some("id"))
    .add_string_field("Name", Some("name"))
    .add_string_field("Email", Some("email"))
    .add_property("Id", "String")
    .add_property("Name", "String")
    .add_property("Email", "String")
    .comment("User model class")
    .build();

// Create a complete unit
let mut unit = DelphiUnit {
    unit_name: "User".to_string(),
    forward_declarations: vec!["TUser".to_string()],
    enums: vec![status_enum],
    records: vec![],
    classes: vec![user_class],
    uses_interface: vec!["System.Classes".to_string(), "System.JSON".to_string()],
    uses_implementation: vec![],
    constants: HashMap::new(),
    comment: Some("User API models".to_string()),
};

// Generate the code
let generator = DelphiCodeGenerator::new(CodeGenConfig::default());
let delphi_code = generator.generate_unit(&unit)?;
```

### Generated Output Example

```pascal
unit uUserModels;

{
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
}

interface

uses
  System.Classes,
  System.JSON;

type
  {$REGION 'Enums and Helpers'}
  {$SCOPEDENUMS ON}
  // __begin_enums__
  // Entity status enumeration
  TStatus = (None, Active, Inactive);

  TStatusHelper = record helper for TStatus
    class function FromString(const pValue: String): TStatus; static;
    function ToString: String;
  end;
  // __end_enums__
  {$SCOPEDENUMS OFF}
  {$ENDREGION}

  {$REGION 'Models'}
  // __begin_classes__
  // User model class
  TUser = class
  strict private:
    // __begin_fields_tuser__
    FId: String;
    FName: String;
    FEmail: String;
    // __end_fields_tuser__
    // Your manual fields go here

  public
    // __begin_properties_tuser__
    property Id: String read FId;
    property Name: String read FName;
    property Email: String read FEmail;
    // __end_properties_tuser__
    // Your manual properties go here

  end;
  // __end_classes__
  {$ENDREGION}

implementation

// Implementation sections with comment markers for safe updates
```

## API Reference

### Core Types

#### `DelphiUnit`
Represents a complete Delphi unit with all its components.

```rust
pub struct DelphiUnit {
    pub unit_name: String,
    pub forward_declarations: Vec<String>,
    pub enums: Vec<DelphiEnum>,
    pub records: Vec<DelphiRecord>,
    pub classes: Vec<DelphiClass>,
    pub uses_interface: Vec<String>,
    pub uses_implementation: Vec<String>,
    pub constants: HashMap<String, String>,
    pub comment: Option<String>,
}
```

#### `DelphiEnum`
Represents a Delphi enumeration with optional helper.

```rust
pub struct DelphiEnum {
    pub name: String,
    pub variants: Vec<DelphiEnumVariant>,
    pub helper_name: String,
    pub generate_helper: bool,
    pub scoped: bool,
    pub comment: Option<String>,
}
```

#### `DelphiClass`
Represents a Delphi class with fields, methods, and properties.

```rust
pub struct DelphiClass {
    pub name: String,
    pub parent_class: Option<String>,
    pub fields: Vec<DelphiField>,
    pub methods: Vec<DelphiMethod>,
    pub properties: Vec<DelphiProperty>,
    pub generate_json_support: bool,
    pub generate_xml_support: bool,
    pub comment: Option<String>,
}
```

#### `DelphiRecord`
Represents a Delphi record type.

```rust
pub struct DelphiRecord {
    pub name: String,
    pub fields: Vec<DelphiField>,
    pub methods: Vec<DelphiMethod>,
    pub generate_json_support: bool,
    pub generate_xml_support: bool,
    pub comment: Option<String>,
}
```

### Builder Patterns

#### `DelphiEnumBuilder`

```rust
let enum_def = DelphiEnumBuilder::new("THttpStatus")
    .add_variant("OK")
    .add_variant_with_value("NotFound", "404")
    .helper_name("THttpStatusHelper")
    .generate_helper(true)
    .comment("HTTP status codes")
    .build();
```

#### `DelphiClassBuilder`

```rust
let class = DelphiClassBuilder::new("TApiResponse")
    .parent_class("TObject")
    .add_string_field("Status", Some("status"))
    .add_reference_field("Data", "TUser", Some("data"))
    .add_property("Status", "String")
    .add_property("Data", "TUser")
    .json_support(true)
    .xml_support(false)
    .comment("API response wrapper")
    .build();
```

### Configuration

#### `CodeGenConfig`

```rust
let config = CodeGenConfig {
    generate_json_helpers: true,
    generate_xml_helpers: false,
    json_helper_unit: "uJsonHelper".to_string(),
    xml_helper_unit: "uXmlHelper".to_string(),
    date_format: "yyyy-mm-dd".to_string(),
    indent_size: 2,
    preserve_existing_code: true,
};
```

### Code Generation and Updates

#### `DelphiCodeGenerator`

```rust
let mut generator = DelphiCodeGenerator::new(config);

// Generate initial unit
let generated_code = generator.generate_unit(&unit)?;

// Update existing unit while preserving manual edits
let updated_code = generator.update_unit(&existing_code, &new_unit)?;

// Selective updates - only update specific sections
let selective_update = generator.update_sections(
    &existing_code,
    vec!["enums".to_string(), "fields_tuser".to_string()],
    &new_unit
)?;
```

#### Update System

The generator uses **comment markers** to identify and preserve sections:

- `// __begin_section__` and `// __end_section__` mark generated code
- Manual additions are preserved after each `// __end_section__` marker
- Updates only replace content within the markers
- Manual code outside markers is completely preserved

## Advanced Features

### 🔄 Intelligent Update System

The generator provides sophisticated update capabilities:

#### Comment-Based Section Markers
- Generated code is wrapped in `// __begin_section__` and `// __end_section__` markers
- Each section (fields, methods, properties, etc.) has unique markers
- Manual additions are preserved between sections

#### Update Modes
1. **Full Update**: Regenerates entire unit while preserving manual edits
2. **Selective Update**: Updates only specified sections
3. **Incremental Update**: Minimal changes with maximum preservation

```rust
// Full update - preserves all manual edits
let updated_code = generator.update_unit(&existing_code, &new_unit)?;

// Selective update - only update enums and specific class fields
let selective_code = generator.update_sections(
    &existing_code,
    vec!["enums".to_string(), "fields_tuser".to_string()],
    &new_unit
)?;
```

### Manual Edit Guidelines

Generated units include comprehensive instructions:

1. **Add manual code AFTER** `// __end_section__` markers
2. **Never modify code INSIDE** the marked sections
3. **Supported manual additions**:
   - Private fields (after `// __end_fields__`)
   - Methods (after `// __end_methods__`)
   - Properties (after `// __end_properties__`)
   - Custom implementations (after `// __end_class_implementation__`)

### JSON Serialization Support

When `generate_json_support` is enabled, the generator automatically creates:

- `FromJson(const pJson: String)` constructor
- `FromJsonRaw(pJson: TJSONValue)` constructor
- Automatic field parsing using JSON keys
- Constants for JSON field names

### XML Serialization Support

When `generate_xml_support` is enabled, the generator creates:

- XML attribute mappings
- XML serialization methods
- XML deserialization methods

### Field Types and Properties

#### Reference vs Value Types

```rust
// Value type (no memory management needed)
.add_string_field("Name", Some("name"))

// Reference type (automatic destructor generation)
.add_reference_field("Profile", "TUserProfile", Some("profile"))
```

#### Property Generation

```rust
// Read-only property
.add_property("Name", "String")

// Property with custom getter/setter
DelphiProperty {
    name: "FullName".to_string(),
    property_type: "String".to_string(),
    getter: Some("GetFullName".to_string()),
    setter: Some("SetFullName".to_string()),
    visibility: DelphiVisibility::Public,
    comment: None,
}
```

### Visibility Levels

```rust
pub enum DelphiVisibility {
    Private,
    StrictPrivate,
    Protected,
    Public,
    Published,
}
```

## Examples

### Complete API Model Generation

```rust
use delphi_code_gen::*;
use std::collections::HashMap;

fn generate_user_api_models() -> Result<String, Box<dyn std::error::Error>> {
    // Create enums
    let status_enum = DelphiEnumBuilder::new("TUserStatus")
        .add_variant("Active")
        .add_variant("Inactive")
        .add_variant("Pending")
        .build();

    // Create address class
    let address_class = DelphiClassBuilder::new("TAddress")
        .add_string_field("Street", Some("street"))
        .add_string_field("City", Some("city"))
        .add_string_field("PostalCode", Some("postalCode"))
        .add_property("Street", "String")
        .add_property("City", "String")
        .add_property("PostalCode", "String")
        .build();

    // Create user class
    let user_class = DelphiClassBuilder::new("TUser")
        .add_string_field("Id", Some("id"))
        .add_string_field("Name", Some("name"))
        .add_string_field("Email", Some("email"))
        .add_reference_field("Address", "TAddress", Some("address"))
        .add_property("Id", "String")
        .add_property("Name", "String")
        .add_property("Email", "String")
        .add_property("Address", "TAddress")
        .build();

    // Create unit
    let unit = DelphiUnit {
        unit_name: "UserApi".to_string(),
        forward_declarations: vec!["TUser".to_string(), "TAddress".to_string()],
        enums: vec![status_enum],
        records: vec![],
        classes: vec![address_class, user_class],
        uses_interface: vec!["System.Classes".to_string(), "System.JSON".to_string()],
        uses_implementation: vec![],
        constants: HashMap::new(),
        comment: Some("User API models".to_string()),
    };

    // Generate code
    let generator = DelphiCodeGenerator::new(CodeGenConfig::default());
    generator.generate_unit(&unit)
}
```

### Record with Methods

```rust
let coordinates_record = DelphiRecord {
    name: "TCoordinates".to_string(),
    fields: vec![
        DelphiField {
            name: "Latitude".to_string(),
            field_type: "Double".to_string(),
            visibility: DelphiVisibility::Public,
            is_reference_type: false,
            json_key: Some("lat".to_string()),
            xml_attribute: Some("latitude".to_string()),
            comment: None,
            default_value: None,
        },
        DelphiField {
            name: "Longitude".to_string(),
            field_type: "Double".to_string(),
            visibility: DelphiVisibility::Public,
            is_reference_type: false,
            json_key: Some("lng".to_string()),
            xml_attribute: Some("longitude".to_string()),
            comment: None,
            default_value: None,
        },
    ],
    methods: vec![DelphiMethod {
        name: "Create".to_string(),
        parameters: vec![
            DelphiParameter {
                name: "pLatitude".to_string(),
                param_type: "Double".to_string(),
                is_const: true,
                is_var: false,
                is_out: false,
                default_value: None,
            },
            DelphiParameter {
                name: "pLongitude".to_string(),
                param_type: "Double".to_string(),
                is_const: true,
                is_var: false,
                is_out: false,
                default_value: None,
            },
        ],
        return_type: Some("TCoordinates".to_string()),
        visibility: DelphiVisibility::Public,
        is_constructor: false,
        is_destructor: false,
        is_class_method: true,
        is_static: true,
        is_virtual: false,
        is_override: false,
        comment: Some("Creates coordinate pair".to_string()),
    }],
    generate_json_support: true,
    generate_xml_support: true,
    comment: Some("Geographic coordinates".to_string()),
};
```

## Update Workflow Example

Here's a typical workflow showing how manual edits are preserved:

### Step 1: Initial Generation
```rust
let generator = DelphiCodeGenerator::new(CodeGenConfig::default());
let initial_code = generator.generate_unit(&unit)?;
```

### Step 2: Add Manual Edits
```pascal
// Developer adds custom code after markers
// __end_fields_tuser__
FMyCustomField: Integer;    // Manual addition
FValidationErrors: TStringList;
```

### Step 3: API Evolution
```rust
// API changes - new fields added to unit definition
let evolved_unit = create_evolved_unit_with_new_fields();

// Update preserves manual edits
let mut generator = DelphiCodeGenerator::new(CodeGenConfig::default());
let updated_code = generator.update_unit(&existing_code, &evolved_unit)?;
```

### Step 4: Result
- Generated fields are updated
- Manual additions (`FMyCustomField`, `FValidationErrors`) are preserved
- Code structure and formatting maintained

## Section Markers Reference

| Section | Marker Pattern | Purpose |
|---------|----------------|---------|
| Forward Declarations | `forward_declarations` | Class forward declarations |
| Enums | `enums` | Enumeration definitions |
| Records | `records` | Record type definitions |
| Classes | `classes` | Class definitions |
| Fields | `fields_{classname}` | Class/Record fields |
| Methods | `methods_{visibility}_{classname}` | Methods by visibility |
| Properties | `properties_{classname}` | Property definitions |
| Implementations | `class_impl_{classname}` | Method implementations |

## Testing

The library includes comprehensive tests for all major functionality:

```bash
cargo test
```

Run the update examples:
```bash
cargo run --bin update_example
```

## Dependencies

- `std::collections::HashMap` for storing key-value pairs
- `std::fmt::Write` for string formatting
- `std::fs` for file operations during updates

## License

This project is licensed under the MIT License - see the LICENSE file for details.