# GitHub Copilot Instructions for XSD Delphi Code Generator

## Project Overview

This is a Rust-based code generator that produces Delphi/Pascal code from XSD (XML Schema) files and OpenAPI specifications. The project is organized as a Cargo workspace with the following modules:

- **cli**: Command-line interface for the code generator
- **codegen**: Core Delphi code generation library with intelligent update capabilities
- **genphi_core**: Shared core functionality including dependency graphs and type registries
- **xml**: XSD parsing and Delphi code generation from XML schemas
- **openapi**: OpenAPI specification parsing and code generation

## Critical Coding Standards

### Error Handling (REQUIRED)

**NEVER use `unwrap()` or `expect()` in production code.** All errors must be handled explicitly.

#### ❌ NEVER DO THIS:
```rust
let file = File::open(path).unwrap();  // FORBIDDEN
let value = option.expect("message");   // FORBIDDEN
```

#### ✅ ALWAYS DO THIS:
```rust
// For Result types - use match, if let, or ? operator
let file = match File::open(path) {
    Ok(f) => f,
    Err(e) => {
        eprintln!("Failed to open file: {e}");
        return Err(ParserError::UnableToReadFile);
    }
};

// Or using if let
let Ok(file) = File::open(path) else {
    return Err(ParserError::UnableToReadFile);
};

// For Option types - use match, if let, or combinators
let value = match option {
    Some(v) => v,
    None => return Err(Error::MissingValue),
};

// Or using if let
let Some(value) = option else {
    return Err(Error::MissingValue);
};
```

#### Exception for test code:
`unwrap()` and `expect()` are acceptable **only** in:
- Test functions (`#[test]`)
- Test examples in documentation comments
- Build scripts (`build.rs`)

### Code Simplicity and Efficiency

1. **Keep public APIs simple**: Public functions should have straightforward signatures and clear semantics, even if this requires more complex internal implementation.

2. **Internal complexity is acceptable**: It's better to have a simple public API with complex internals than a complex API with simple internals.

3. **Favor clarity over cleverness**: Write code that is easy to understand and maintain. Avoid obscure Rust tricks unless they provide significant value.

4. **Performance matters**: While clarity is important, don't sacrifice efficiency unnecessarily. Use appropriate data structures and algorithms.

#### Example:
```rust
// ✅ Good: Simple public API
pub fn parse_file<P: AsRef<Path>>(
    &mut self,
    path: P,
    registry: &mut TypeRegistry<CustomTypeDefinition>,
) -> Result<ParsedData, ParserError> {
    // Complex internal implementation is fine
    let Ok(mut reader) = Reader::from_file(path) else {
        return Err(ParserError::UnableToReadFile);
    };
    self.parse_nodes(&mut reader, registry)
}

// ❌ Bad: Complex public API
pub fn parse_file_with_reader_and_options(
    &mut self,
    reader: &mut Reader<BufReader<File>>,
    registry: &mut TypeRegistry<CustomTypeDefinition>,
    options: ParseOptions,
    callbacks: Vec<Box<dyn Fn(&Event)>>,
) -> Result<ParsedData, ParserError> {
    // Simple internals but confusing API
}
```

## Code Style Guidelines

### Documentation

- **Always document public APIs**: Use `///` doc comments for all public items
- **Include examples**: Provide usage examples in documentation when helpful
- **Document errors**: Explain what errors can be returned and why
- **Keep it concise**: Documentation should be clear and to the point

```rust
/// Parses a single XML file into structured data.
///
/// Returns parsed XSD schema data that can be used for code generation.
/// The `TypeRegistry` tracks all custom types discovered during parsing.
///
/// # Arguments
///
/// * `path` - Path to the XSD file to parse
/// * `registry` - Type registry for storing discovered types
///
/// # Errors
///
/// Returns `ParserError::UnableToReadFile` if the file cannot be opened or read.
/// Returns `ParserError::InvalidXml` if the XML structure is malformed.
///
/// # Example
///
/// ```rust
/// let mut parser = XmlParser::default();
/// let mut registry = TypeRegistry::new();
/// // Method call - &mut self is implicit via parser instance
/// let data = parser.parse_file("schema.xsd", &mut registry)?;
/// ```
pub fn parse_file<P: AsRef<Path>>(
    &mut self,
    path: P,
    registry: &mut TypeRegistry<CustomTypeDefinition>,
) -> Result<ParsedData, ParserError>
```

### Naming Conventions

- **Use descriptive names**: Variable and function names should clearly indicate their purpose
- **Follow Rust conventions**: 
  - `snake_case` for functions, variables, modules
  - `PascalCase` for types, traits, enums
  - `SCREAMING_SNAKE_CASE` for constants
  - `'lowercase` for lifetimes

### Pattern Matching

- **Prefer exhaustive matches**: Use exhaustive pattern matching when possible to catch new enum variants
- **Use `if let` for single patterns**: When matching a single pattern, `if let` is more concise
- **Use early returns**: Guard clauses and early returns improve readability

```rust
// ✅ Good: Exhaustive match
match node_type {
    NodeType::Standard(base) => handle_standard(base),
    NodeType::Custom(name) => handle_custom(name),
}

// ✅ Good: if let for single pattern
if let Some(documentation) = node.documentations {
    process_docs(documentation);
}

// ✅ Good: Early return
let Some(base_type) = st.base_type.as_ref() else {
    return Err(Error::MissingBaseType);
};
```

### Type Safety

- **Use strong types**: Prefer newtype patterns and custom types over primitives when it adds clarity
- **Avoid string-based identifiers** where enums or custom types would be clearer
- **Use `const` when possible**: Mark functions and constructors as `const` when they can be evaluated at compile time

### Templates and Code Generation

When working with Tera templates for Delphi code generation:

- **Keep templates maintainable**: Break large templates into smaller, reusable macros
- **Validate template context**: Ensure all required variables are present in the context
- **Handle template errors gracefully**: Don't panic on template errors; return proper error types

```rust
// ✅ Good: Proper template error handling
let mut tera = Tera::default();
if let Err(e) = tera.add_raw_templates(templates) {
    return Err(CodeGenError::TemplateEngineError(
        format!("Failed to load templates: {e:?}")
    ));
}
```

## Common Patterns in This Codebase

### Type Registry Pattern

The project uses a `TypeRegistry` to track custom types during parsing:

```rust
let mut type_registry = TypeRegistry::<CustomTypeDefinition>::new();
parser.parse_file(&path, &mut type_registry)?;
```

### Dependency Graph

Complex type dependencies are resolved using `DependencyGraph`:

```rust
let mut graph = DependencyGraph::new();
graph.add(item);
let sorted = graph.build_dependency_sorted_list();
```

### Internal Representation

XSD/OpenAPI structures are converted to an intermediate representation before code generation:

```rust
let ir = InternalRepresentation::build(&parsed_data, &type_registry);
let generator = DelphiCodeGenerator::new(writer, options, ir, docs);
generator.generate()?;
```

### Builder Pattern

Use builder pattern for complex type construction:

```rust
let class = DelphiClassBuilder::new("TUser")
    .add_string_field("Id", Some("id"))
    .add_property("Name", "String")
    .comment("User model")
    .build();
```

## Testing

- **Write meaningful tests**: Tests should verify behavior, not implementation
- **Use descriptive test names**: Test function names should describe what they test
- **Test error cases**: Don't just test the happy path; verify error handling
- **Use `pretty_assertions`**: Available in dev dependencies for better assertion output

```rust
#[test]
fn test_parse_simple_type_with_restrictions() {
    let xml = r#"<xs:simpleType name="Age">...</xs:simpleType>"#;
    let mut parser = XmlParser::default();
    let mut registry = TypeRegistry::new();
    
    let result = parser.parse(xml, &mut registry).unwrap(); // OK in tests
    assert_eq!(result.nodes.len(), 1);
}
```

## Performance Considerations

- **Avoid unnecessary allocations**: Reuse buffers and collections when possible
- **Use appropriate data structures**: HashMap for lookups, Vec for ordered data
- **Consider using `Cow`** for borrowed vs. owned data when it reduces allocations
- **Profile before optimizing**: Don't optimize prematurely

## Module Organization

When adding new functionality:

1. **Keep modules focused**: Each module should have a single, clear responsibility
2. **Use `mod.rs` for module organization**: Re-export public items in `mod.rs`
3. **Minimize public API surface**: Only expose what consumers need
4. **Internal helpers go in `helper.rs`**: Shared utility functions

## Dependencies

- **Minimize external dependencies**: Only add new dependencies when necessary
- **Prefer well-maintained crates**: Use popular, actively maintained libraries
- **Document why dependencies exist**: Add comments for non-obvious dependencies

## CLI Design

The CLI should be user-friendly:

- **Clear error messages**: Tell users what went wrong and how to fix it
- **Validate inputs early**: Check arguments before doing expensive operations
- **Provide helpful defaults**: Sensible defaults reduce CLI complexity
- **Support both relative and absolute paths**: Make the tool easy to use

## When to Ask for Help

If you encounter:
- **Ambiguous requirements**: Ask for clarification rather than guessing
- **Complex architectural decisions**: Discuss before implementing major changes
- **Performance bottlenecks**: Profile first, then discuss optimization strategies
- **Breaking changes**: Discuss impact and migration path

## Summary of Key Rules

1. ✅ **NEVER use `unwrap()` or `expect()` in production code**
2. ✅ **Always handle errors explicitly with `Result` and `Option`**
3. ✅ **Keep public APIs simple; internal complexity is acceptable**
4. ✅ **Write clear, efficient code that is easy to maintain**
5. ✅ **Document all public items with examples**
6. ✅ **Follow Rust naming conventions and idioms**
7. ✅ **Test thoroughly, including error cases**
8. ✅ **Profile before optimizing for performance**
