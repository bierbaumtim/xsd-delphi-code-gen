# XSD Delphi Code Generator
## Usage
**Single File**
`xsd-delphi-code-gen -i test.xsd -o test.pas --unit-name test`

**Multiple Files**
`xsd-delphi-code-gen -i test.xsd -i types.xsd -o test.pas --unit-name test`

## Supported Features
- Namespaces
- Union Types
- Enumerations
- Simple Types as TypeAlias or Derived Type
- Complex Type 
  - Inheritance
  - ComplexContent
  - Sequence
- Built-In DataTypes (string, boolean, decimal, float, double, dateTime, time, date, hexBinary, base64Binary)
- Built-I derived DataTypes (Integer, nonPositiveInteger, negativeInteger, long, int, short, byte, nonNegativeInteger, unsignedLong, unsignedInt, unsignedShort, unsignedByte, positiveInteger)

## Planned support
- Attributes
- xs:choice for xs:complexType
- xs:element->nillable(xs:nil)
- xs:list -> currently partially supported

## Not planned
- References
- Groups (xs:group)
- Any (xs:any)
- AnyAttribute
- AttributeGroup(?)
- xs:pattern