{% import "macros.pas" as macros %}

{%- set timestamp = now() | date(format="%d.%m.%Y %H:%m:%S") -%}
// ========================================================================== //
// Generated by Delphi Code Gen - Mode XSD2Delphi                             //
// {{ macros::fixed_size_line(content="Version: " ~ crate_version, size=74) }} //
// {{ macros::fixed_size_line(content="Timestamp: " ~ timestamp, size=74) }} //
// ========================================================================== //
{% for line in documentations -%}
// {{line}}
{%- endfor %}

unit {{unitName}};

interface

uses System.DateUtils,
     System.Generics.Collections,
     System.Net.URLClient,
     System.Types,
     System.TypInfo,
     System.StrUtils,
     System.SysUtils,
     Xml.XMLDoc,
     Xml.XMLIntf;

type
  {$REGION 'Optional Helper'}
  TOptional<T> = class abstract
  strict protected
    FOwns: Boolean;
  public
    function Unwrap: T; virtual;
    function UnwrapOr(pDefault: T): T; virtual; abstract;
    function IsSome: Boolean; virtual; abstract;
    function IsNone: Boolean; virtual; abstract;
    function CopyWith(pValue: T): TOptional<T>; virtual; abstract;

    property Owns: Boolean read FOwns write FOwns;
  end;

  TSome<T> = class sealed(TOptional<T>)
  strict private
    FValue: T;
  public
    constructor Create(pValue: T);
    destructor Destroy; override;

    function Unwrap: T; override;
    function UnwrapOr(pDefault: T): T; override;
    function IsSome: Boolean; override;
    function IsNone: Boolean; override;
    function CopyWith(pValue: T): TOptional<T>; override;
  end;

  TNone<T> = class sealed(TOptional<T>)
  public
    function UnwrapOr(pDefault: T): T; override;
    function IsSome: Boolean; override;
    function IsNone: Boolean; override;
    function CopyWith(pValue: T): TOptional<T>; override;
  end;
  {$ENDREGION}

  {% if enumerations | length > 0 -%}
  {$REGION 'Enumerations'}
  {%- for enum in enumerations %}
  // XML Qualified Name: {{enum.qualified_name}}
  {% for line in enum.documentations -%}
  // {{line}}
  {% endfor -%}
  {% if enum.line_per_variant -%}
  {{enum.name}} = (
  {%- for value in enum.values %}
  {%- for line in value.documentations %}
  // {{line}}
  {% endfor -%}
  {{value.variant_name}}
  {%- if not loop.last -%}{{","}}{%- endif -%}
  {% endfor -%}
  );
  {% else -%}
  {{enum.name}} = ({{enum.values | map(attribute="variant_name") | join(sep=", ")}});
  {% endif -%}
  {% endfor -%}
  {$ENDREGION}

  {$REGION 'Enumerations Helper'}
  {%- for enum in enumerations %}
  {{enum.name}}Helper = record helper for {{enum.name}}
  {%- if gen_from_xml %}
    class function FromXmlValue(const pXmlValue: String): {{enum.name}}; static;
  {%- endif %}
  {%- if gen_to_xml %}
    function ToXmlValue: String;
  {%- endif %}
  end;
  {% endfor -%}
  {$ENDREGION}
  {%- endif %}

  {% if classes | length > 0 -%}
  {$REGION 'Forward Declarations}
  {{""}}{# Requried to get a newline here #}
  {%- for class in classes -%}
  {{class.name}} = class;
  {% endfor -%}
  {$ENDREGION}
  {%- endif %}

  {% if type_aliases | length > 0 -%}
  {$REGION 'Aliases'}
  {%- for alias in type_aliases %}
  // XML Qualified Name: {{alias.qualified_name}}
  {% for line in alias.documentations -%}
  // {{line}}
  {% endfor -%}
  {{alias.name}} = {{alias.data_type_repr}};
  {% endfor -%}
  {$ENDREGION}
  {%- endif %}

  {$REGION 'Declarations}
  {{ macros::class_declaration(class=document) }}
  {{""}}
  {%- for class in classes %}
  {{ macros::class_declaration(class=class) }}
  {% endfor -%}
  {$ENDREGION}

  {%- if union_types | length > 0 %}
  {$REGION 'Union Types'}
  {%- for union in union_types %}
    // XML Qualified Name: {{alias.qualified_name}}
    {% for line in union.documentations -%}
    // {{line}}
    {% endfor -%}
    {{union.name}} = record
      type Variants = ({{union.variants | map(attribute="name") | join(sep=", ")}});

      case Variant: Variants of
      {% for variant in union.variants %}
        Variants.{{variant.name}}: {{variant.variable_name}}: {{variant.data_type_repr}};
      {% endfor %}
      end;
    end;
  {% endfor -%}
  {$ENDREGION}

  {$REGION 'Union Types Helper'}
  {%- for union in union_types %}
  {{union.name}}Helper = record helper for {{union.name}}
  {%- if gen_from_xml %}
    class function FromXml(node: IXMLNode): {{union.name}}; static;
  {%- endif %}
  {%- if gen_to_xml %}
    function ToXmlValue: String;
  {%- endif %}
  end;
  {% endfor -%}
  {$ENDREGION}
  {%- endif %}

implementation
{% if needs_net_encoding_unit_use_clause -%}
uses System.NetEncoding;
{%- endif %}

const
  cnXmlTrueValue: string = 'true';
  cnXmlFalseValue: string = 'false';

{% if gen_datetime_helper or gen_hex_binary_helper -%}
{$REGION 'Helper'}
{% if gen_datetime_helper and gen_from_xml -%}
function DecodeDateTime(const pDateStr: String; const pFormat: String = ''): TDateTime;
begin
  if pFormat = '' then Exit(ISO8601ToDate(pDateStr));

  Result := ISO8601ToDate(pDateStr);
end;
{%- endif %}

{% if gen_datetime_helper and gen_to_xml  -%}
function EncodeTime(const pTime: TTime; const pFormat: String): String;
begin
  var vFormatSettings := TFormatSettings.Create;
  vFormatSettings.LongTimeFormat := pFormat;

  Result := TimeToStr(pTime, vFormatSettings);
end;
{%- endif %}

{% if gen_hex_binary_helper and gen_from_xml -%}
function HexStrToBin(const pHex: String): TBytes;
begin
  HexToBin(pHex, 0, Result, 0, Length(pHex) / 2);
end;
{%- endif %}

{% if gen_hex_binary_helper and gen_to_xml -%}
function BinToHexStr(const pBin: TBytes): String;
begin
  var vTemp: TBytes;
  BinToHex(pBin, 0, vTemp, Length(pBin));

  Result := TEncoding.GetString(vTemp);
end;
{%- endif %}
{$ENDREGION}
{%- endif %}

{% if enumerations | length > 0 -%}
{$REGION 'Enumerations Helper'}
{%- for enum in enumerations %}
{%- if gen_from_xml %}
class function {{enum.name}}Helper.FromXmlValue(const pXmlValue: String): {{enum.name}};
begin
  {{""}} {# Required to get newline between first if and the function begin #}
  {%- for value in enum.values %}
  {%- if loop.first -%}
  if pXmlValue = '{{value.xml_value}}' then begin
  {%- else -%}
  {{" if"}} pXmlValue = '{{value.xml_value}}' then begin
  {%- endif %}
    Result := {{enum.name}}.{{value.variant_name}};
  end else
  {%- endfor %} begin
    raise Exception.Create('\"' + pXmlValue + '\" is a unknown value for {{enum.name}}');
  end;
end;
{%- endif %}

{% if gen_to_xml -%}
function {{enum.name}}Helper.ToXmlValue: String;
begin
  case Self of
    {%- for value in enum.values %}
    {{enum.name}}.{{value.variant_name}}: Result := '{{value.xml_value}}';
    {%- endfor %}
  end;
end;
{%- endif %}
{% endfor -%}
{$ENDREGION}
{%- endif %}

{$REGION 'Declarations}
{{  macros::class_implementation(class=document)  }}
{{""}}
{%- for class in classes %}
{{  macros::class_implementation(class=class)  }}
{% endfor -%}
{$ENDREGION}

{%- if union_types | length > 0 %}
{$REGION 'Union Types Helper'}
{%- for union in union_types %}
{{union.name}}Helper = record helper for {{union.name}}
{%- if gen_from_xml %}
class function {{union.name}}Helper.FromXml(node: IXMLNode): {{union.name}};
begin
  // TODO: CodeGen for this is currently not supported. Manual implementation required
end;
{%- endif %}
{%- if gen_to_xml %}
function {{union.name}}Helper.ToXmlValue: String;
begin
  case Self.Variant of
  {% for variant in union.variants %}
    {% if variant.is_list_type %}
    Variants.{{variant.name}}: Result := ""; // TODO: CodeGen for this type is currently not supported. Manual implementation required",
    {% elif variant.is_inline_list %}
    Variants.{{variant.name}}: begin
      Result := '';

      for var I := Low({{variant.variable_name}}) to High({{variant.variable_name}}) do begin
        Result := Result + {{variant.value_as_str_repr}};

        if I < High({{variant.variable_name}}) then begin
          Result := Result + ' ';
        end;
      end;
    end;
    {% elif variant.use_to_xml_func %}
    Variants.{{variant.name}}: Result := {{variant.variable_name}}.ToXmlValue;
    {% else %}
    Variants.{{variant.name}}: Result := {{variant.value_as_str_repr}};
    {%- endif %}
  {% endfor %}
  end;
end;
{%- endif %}
end;
{%- endfor %}
{$ENDREGION}
{%- endif %}

{$REGION 'Optional Helper'}
{ TOptional<T> }
function TOptional<T>.Unwrap: T;
begin
  raise Exception.Create('Not Implemented');
end;

{ TSome<T> }
constructor TSome<T>.Create(pValue: T);
begin
  FValue := pValue;
end;

function TSome<T>.IsNone: Boolean;
begin
  Result := False;
end;

function TSome<T>.IsSome: Boolean;
begin
  Result := True;
end;

function TSome<T>.Unwrap: T;
begin
  Result := FValue;
end;

function TSome<T>.UnwrapOr(pDefault: T): T;
begin
  Result := FValue;
end;

function TSome<T>.CopyWith(pValue: T): TOptional<T>;
begin
  FValue := pValue;
  Result := Self;
end;

destructor TSome<T>.Destroy;
begin
  if FOwns then begin
    if PTypeInfo(TypeInfo(T)).Kind = tkClass then begin
      PObject(@FValue).Free;
    end;
  end;
end;

{ TNone<T> }
function TNone<T>.IsNone: Boolean;
begin
  Result := True;
end;

function TNone<T>.IsSome: Boolean;
begin
  Result := False;
end;

function TNone<T>.UnwrapOr(pDefault: T): T;
begin
  Result := pDefault;
end;

function TNone<T>.CopyWith(pValue: T): TOptional<T>;
begin
  Result := TSome<T>.Create(pValue);
  Self.Free;
end;
{$ENDREGION}

end.