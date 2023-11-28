use std::io::Write;

use super::code_writer::CodeWriter;

/// Generates a Delphi implementation for Option
pub struct OptionalCodeGenerator;

impl OptionalCodeGenerator {
    pub fn write_declarations<T: Write>(
        writer: &mut CodeWriter<T>,
        indentation: usize,
    ) -> Result<(), std::io::Error> {
        writer.writeln("{$REGION 'Optional Helper'}", Some(indentation))?;
        writer.writeln("TOptional<T> = class abstract", Some(indentation))?;
        writer.writeln("strict protected", Some(indentation))?;
        writer.writeln("FOwns: Boolean;", Some(indentation + 2))?;
        writer.writeln("public", Some(indentation))?;
        writer.writeln("function Unwrap: T; virtual;", Some(indentation + 2))?;
        writer.writeln(
            "function UnwrapOr(pDefault: T): T; virtual; abstract;",
            Some(indentation + 2),
        )?;
        writer.writeln(
            "function IsSome: Boolean; virtual; abstract;",
            Some(indentation + 2),
        )?;
        writer.writeln(
            "function IsNone: Boolean; virtual; abstract;",
            Some(indentation + 2),
        )?;
        writer.writeln(
            "function CopyWith(pValue: T): TOptional<T>; virtual; abstract;",
            Some(indentation + 2),
        )?;
        writer.newline()?;
        writer.writeln(
            "property Owns: Boolean read FOwns write FOwns;",
            Some(indentation + 2),
        )?;
        writer.writeln("end;", Some(indentation))?;
        writer.newline()?;

        writer.writeln("TSome<T> = class sealed(TOptional<T>)", Some(indentation))?;
        writer.writeln("strict private", Some(indentation))?;
        writer.writeln("FValue: T;", Some(indentation + 2))?;
        writer.writeln("public", Some(indentation))?;
        writer.writeln("constructor Create(pValue: T);", Some(indentation + 2))?;
        writer.writeln("destructor Destroy; override;", Some(indentation + 2))?;
        writer.newline()?;
        writer.writeln("function Unwrap: T; override;", Some(indentation + 2))?;
        writer.writeln(
            "function UnwrapOr(pDefault: T): T; override;",
            Some(indentation + 2),
        )?;
        writer.writeln("function IsSome: Boolean; override;", Some(indentation + 2))?;
        writer.writeln("function IsNone: Boolean; override;", Some(indentation + 2))?;
        writer.writeln(
            "function CopyWith(pValue: T): TOptional<T>; override;",
            Some(indentation + 2),
        )?;
        writer.writeln("end;", Some(indentation))?;
        writer.newline()?;

        writer.writeln("TNone<T> = class sealed(TOptional<T>)", Some(indentation))?;
        writer.writeln("public", Some(indentation))?;
        writer.writeln(
            "function UnwrapOr(pDefault: T): T; override;",
            Some(indentation + 2),
        )?;
        writer.writeln("function IsSome: Boolean; override;", Some(indentation + 2))?;
        writer.writeln("function IsNone: Boolean; override;", Some(indentation + 2))?;
        writer.writeln(
            "function CopyWith(pValue: T): TOptional<T>; override;",
            Some(indentation + 2),
        )?;
        writer.writeln("end;", Some(indentation))?;
        writer.writeln("{$ENDREGION}", Some(indentation))?;

        Ok(())
    }

    pub fn write_implementationw<T: Write>(
        writer: &mut CodeWriter<T>,
    ) -> Result<(), std::io::Error> {
        writer.writeln("{$REGION 'Optional Helper'}", None)?;

        writer.writeln("{ TOptional<T> }", None)?;
        writer.writeln("function TOptional<T>.Unwrap: T;", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("raise Exception.Create('Not Implemented');", Some(2))?;
        writer.writeln("end;", None)?;
        writer.newline()?;

        writer.writeln("{ TSome<T> }", None)?;
        writer.writeln("constructor TSome<T>.Create(pValue: T);", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("FValue := pValue;", Some(2))?;
        writer.writeln("end;", None)?;
        writer.newline()?;
        writer.writeln("function TSome<T>.IsNone: Boolean;", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("Result := False;", Some(2))?;
        writer.writeln("end;", None)?;
        writer.newline()?;
        writer.writeln("function TSome<T>.IsSome: Boolean;", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("Result := True;", Some(2))?;
        writer.writeln("end;", None)?;
        writer.newline()?;
        writer.writeln("function TSome<T>.Unwrap: T;", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("Result := FValue;", Some(2))?;
        writer.writeln("end;", None)?;
        writer.newline()?;
        writer.writeln("function TSome<T>.UnwrapOr(pDefault: T): T;", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("Result := FValue;", Some(2))?;
        writer.writeln("end;", None)?;
        writer.newline()?;
        writer.writeln("function TSome<T>.CopyWith(pValue: T): TOptional<T>;", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("FValue := pValue;", Some(2))?;
        writer.writeln("Result := Self;", Some(2))?;
        writer.writeln("end;", None)?;
        writer.newline()?;
        writer.writeln("destructor TSome<T>.Destroy;", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("if FOwns then begin", Some(2))?;
        writer.writeln(
            "if PTypeInfo(TypeInfo(T)).Kind = tkClass then begin",
            Some(4),
        )?;
        writer.writeln("PObject(@FValue).Free;", Some(6))?;
        writer.writeln("end;", Some(4))?;
        writer.writeln("end;", Some(2))?;
        writer.writeln("end;", None)?;
        writer.newline()?;

        writer.writeln("{ TNone<T> }", None)?;
        writer.writeln("function TNone<T>.IsNone: Boolean;", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("Result := True;", Some(2))?;
        writer.writeln("end;", None)?;
        writer.newline()?;
        writer.writeln("function TNone<T>.IsSome: Boolean;", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("Result := False;", Some(2))?;
        writer.writeln("end;", None)?;
        writer.newline()?;
        writer.writeln("function TNone<T>.UnwrapOr(pDefault: T): T;", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("Result := pDefault;", Some(2))?;
        writer.writeln("end;", None)?;
        writer.newline()?;
        writer.writeln("function TNone<T>.CopyWith(pValue: T): TOptional<T>;", None)?;
        writer.writeln("begin", None)?;
        writer.writeln("Result := TSome<T>.Create(pValue);", Some(2))?;
        writer.writeln("Self.Free;", Some(2))?;
        writer.writeln("end;", None)?;

        writer.writeln("{$ENDREGION}", None)?;
        Ok(())
    }
}
