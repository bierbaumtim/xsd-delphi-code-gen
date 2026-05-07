use std::fs;
use std::io::BufWriter;

use genphi_core::type_registry::TypeRegistry;
use xml::generator::code_generator_trait::{CodeGenOptions, CodeGenerator};
use xml::generator::delphi::code_generator::DelphiCodeGenerator;
use xml::generator::internal_representation::InternalRepresentation;
use xml::parser::types::CustomTypeDefinition;
use xml::parser::xml::XmlParser;

#[test]
fn test_nillable_attribute_parses_and_generates() {
    let xsd = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="TestElement" type="xs:string" nillable="true"/>
  <xs:element name="Root">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="nullable" type="xs:string" nillable="true"/>
        <xs:element name="required" type="xs:string"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

    let tmp_file = "/tmp/test_nillable.xsd";
    fs::write(tmp_file, xsd).expect("Failed to write test XSD");

    let mut parser = XmlParser::default();
    let mut registry = TypeRegistry::<CustomTypeDefinition>::new();

    let data = parser
        .parse_file(tmp_file, &mut registry)
        .expect("Failed to parse XSD with nillable");

    // Check that nillable attribute is parsed
    assert!(data.nodes.len() >= 1, "Should have parsed elements");

    let ir = InternalRepresentation::build(&data, &registry);

    let output = Vec::new();
    let buffer = BufWriter::new(output);
    let options = CodeGenOptions {
        unit_name: "Test".to_string(),
        type_prefix: None,
        generate_from_xml: true,
        generate_to_xml: true,
        enable_validation: false,
        xsd_file_paths: vec![],
    };

    let mut generator = DelphiCodeGenerator::new(buffer, options, ir, data.documentations);

    // Test that generation succeeds
    generator
        .generate()
        .expect("Failed to generate code from nillable XSD");

    fs::remove_file(tmp_file).ok();
}

#[test]
fn test_list_type_parses_and_generates() {
    let xsd = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:simpleType name="StringList">
    <xs:list itemType="xs:string"/>
  </xs:simpleType>
  <xs:simpleType name="IntegerList">
    <xs:list itemType="xs:integer"/>
  </xs:simpleType>
  <xs:element name="Root">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="strings" type="StringList"/>
        <xs:element name="numbers" type="IntegerList"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

    let tmp_file = "/tmp/test_list.xsd";
    fs::write(tmp_file, xsd).expect("Failed to write test XSD");

    let mut parser = XmlParser::default();
    let mut registry = TypeRegistry::<CustomTypeDefinition>::new();

    let data = parser
        .parse_file(tmp_file, &mut registry)
        .expect("Failed to parse XSD with list types");

    // Verify list types were registered
    assert!(
        registry.types.contains_key("StringList"),
        "StringList should be registered"
    );
    assert!(
        registry.types.contains_key("IntegerList"),
        "IntegerList should be registered"
    );

    let ir = InternalRepresentation::build(&data, &registry);

    // Verify list types are in type aliases
    assert!(
        !ir.types_aliases.is_empty(),
        "Should have type aliases for list types"
    );

    let output = Vec::new();
    let buffer = BufWriter::new(output);
    let options = CodeGenOptions {
        unit_name: "Test".to_string(),
        type_prefix: None,
        generate_from_xml: true,
        generate_to_xml: true,
        enable_validation: false,
        xsd_file_paths: vec![],
    };

    let mut generator = DelphiCodeGenerator::new(buffer, options, ir, data.documentations);

    // Test that generation succeeds
    generator
        .generate()
        .expect("Failed to generate code from list types XSD");

    fs::remove_file(tmp_file).ok();
}

#[test]
fn test_choice_parses_and_generates() {
    let xsd = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="ChoiceType">
    <xs:choice>
      <xs:element name="option1" type="xs:string"/>
      <xs:element name="option2" type="xs:integer"/>
      <xs:element name="option3" type="xs:boolean"/>
    </xs:choice>
  </xs:complexType>
  <xs:element name="Root" type="ChoiceType"/>
</xs:schema>"#;

    let tmp_file = "/tmp/test_choice.xsd";
    fs::write(tmp_file, xsd).expect("Failed to write test XSD");

    let mut parser = XmlParser::default();
    let mut registry = TypeRegistry::<CustomTypeDefinition>::new();

    let data = parser
        .parse_file(tmp_file, &mut registry)
        .expect("Failed to parse XSD with choice");

    // Verify ChoiceType was registered
    assert!(
        registry.types.contains_key("ChoiceType"),
        "ChoiceType should be registered"
    );

    let ir = InternalRepresentation::build(&data, &registry);

    // Verify class was created for choice type
    assert!(!ir.classes.is_empty(), "Should have class for choice type");

    let output = Vec::new();
    let buffer = BufWriter::new(output);
    let options = CodeGenOptions {
        unit_name: "Test".to_string(),
        type_prefix: None,
        generate_from_xml: true,
        generate_to_xml: true,
        enable_validation: false,
        xsd_file_paths: vec![],
    };

    let mut generator = DelphiCodeGenerator::new(buffer, options, ir, data.documentations);

    // Test that generation succeeds
    generator
        .generate()
        .expect("Failed to generate code from choice XSD");

    fs::remove_file(tmp_file).ok();
}

#[test]
fn test_combined_features_parses_and_generates() {
    let xsd = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:simpleType name="Tags">
    <xs:list itemType="xs:string"/>
  </xs:simpleType>
  
  <xs:complexType name="OptionType">
    <xs:choice>
      <xs:element name="textOption" type="xs:string"/>
      <xs:element name="numberOption" type="xs:integer"/>
    </xs:choice>
  </xs:complexType>
  
  <xs:element name="Item">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="id" type="xs:string"/>
        <xs:element name="tags" type="Tags"/>
        <xs:element name="description" type="xs:string" nillable="true"/>
        <xs:element name="choice" type="OptionType"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

    let tmp_file = "/tmp/test_combined.xsd";
    fs::write(tmp_file, xsd).expect("Failed to write test XSD");

    let mut parser = XmlParser::default();
    let mut registry = TypeRegistry::<CustomTypeDefinition>::new();

    let data = parser
        .parse_file(tmp_file, &mut registry)
        .expect("Failed to parse combined features XSD");

    // Verify all types were registered
    assert!(
        registry.types.contains_key("Tags"),
        "Tags list type should be registered"
    );
    assert!(
        registry.types.contains_key("OptionType"),
        "OptionType should be registered"
    );

    let ir = InternalRepresentation::build(&data, &registry);

    // Verify generated structures
    assert!(!ir.types_aliases.is_empty(), "Should have type aliases");
    assert!(!ir.classes.is_empty(), "Should have classes");

    let output = Vec::new();
    let buffer = BufWriter::new(output);
    let options = CodeGenOptions {
        unit_name: "Test".to_string(),
        type_prefix: None,
        generate_from_xml: true,
        generate_to_xml: true,
        enable_validation: false,
        xsd_file_paths: vec![],
    };

    let mut generator = DelphiCodeGenerator::new(buffer, options, ir, data.documentations);

    // Test that generation succeeds with all features combined
    generator
        .generate()
        .expect("Failed to generate code from combined features XSD");

    fs::remove_file(tmp_file).ok();
}

#[test]
fn test_validation_code_generation() {
    let xsd = r#"<?xml version="1.0" encoding="utf-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:element name="Person">
        <xs:complexType>
            <xs:sequence>
                <xs:element name="Name" type="xs:string"/>
                <xs:element name="Age" type="xs:int"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
</xs:schema>"#;

    let tmp_file = "/tmp/test_validation.xsd";
    fs::write(tmp_file, xsd).expect("Failed to write test XSD");

    let mut parser = XmlParser::default();
    let mut registry = TypeRegistry::<CustomTypeDefinition>::new();

    let data = parser
        .parse_file(tmp_file, &mut registry)
        .expect("Failed to parse validation test XSD");

    let ir = InternalRepresentation::build(&data, &registry);

    // Test with validation enabled
    let output = Vec::new();
    let buffer = BufWriter::new(output);
    let options_with_validation = CodeGenOptions {
        unit_name: "TestValidation".to_string(),
        type_prefix: None,
        generate_from_xml: true,
        generate_to_xml: true,
        enable_validation: true,
        xsd_file_paths: vec![tmp_file.into()],
    };

    let mut generator = DelphiCodeGenerator::new(
        buffer,
        options_with_validation,
        ir,
        data.documentations.clone(),
    );

    generator
        .generate()
        .expect("Failed to generate code with validation");

    // Test without validation enabled
    let output2 = Vec::new();
    let buffer2 = BufWriter::new(output2);
    let options_without_validation = CodeGenOptions {
        unit_name: "TestNoValidation".to_string(),
        type_prefix: None,
        generate_from_xml: true,
        generate_to_xml: true,
        enable_validation: false,
        xsd_file_paths: vec![],
    };

    let ir2 = InternalRepresentation::build(&data, &registry);
    let mut generator2 = DelphiCodeGenerator::new(
        buffer2,
        options_without_validation,
        ir2,
        data.documentations,
    );

    generator2
        .generate()
        .expect("Failed to generate code without validation");

    fs::remove_file(tmp_file).ok();
}
