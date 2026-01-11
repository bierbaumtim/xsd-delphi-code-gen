# Delphi Code Generator
## Usage
**Single File**
`genphi -i test.xsd -o test.pas --unit-name test --mode xml`

**Multiple Files**
`genphi -i test.xsd -i types.xsd -o test.pas --unit-name test --mode xml`

## XML Support
### Supported Features
- Namespaces
- Union Types
- Enumerations
- Simple Types as TypeAlias or Derived Type
- Complex Type 
  - Inheritance
  - ComplexContent
  - Sequence
  - Choice (xs:choice for xs:complexType)
- Attributes
- Nillable elements (xs:element->nillable)
- List types (xs:list)
- Built-In DataTypes (string, boolean, decimal, float, double, dateTime, time, date, hexBinary, base64Binary)
- Built-In derived DataTypes (Integer, nonPositiveInteger, negativeInteger, long, int, short, byte, nonNegativeInteger, unsignedLong, unsignedInt, unsignedShort, unsignedByte, positiveInteger)

### Planned support
(None currently)

### Not planned
- References
- Groups (xs:group)
- Any (xs:any)
- AnyAttribute
- AttributeGroup(?)
- xs:pattern