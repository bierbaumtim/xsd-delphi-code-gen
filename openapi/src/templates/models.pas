{% macro fixed_size_line(content, size) %}
  {%- set content_length = content | length -%}
  {%- set space_count = size - content_length -%}
  {{ content }}
  {%- for v in range(end=space_count) -%}
  {{" "}}
  {%- endfor -%}
{% endmacro fixed_size_line %}

{% macro type_name(base_type, is_list_type, is_reference_type) %}
  {%- if is_list_type and is_reference_type -%}
  TObjectList<{{base_type}}>
  {%- elif is_list_type -%}
  TList<{{base_type}}>
  {%- else -%}
  {{base_type}}
  {%- endif -%}
{% endmacro type_name %}

{% macro from_json(base_type, is_list_type, is_reference_type, is_enum_type, key) %}
  {%- if is_list_type and is_reference_type -%}
  TJsonHelper.DeserializeObjectList<T{{prefix}}{{base_type}}>(
    vRoot.GetValue<TJSONArray>({{key}}),
    function (pJson: TJSONValue): T{{prefix}}{{base_type}}
    begin
      Result := T{{prefix}}{{base_type}}.FromJsonRaw(pJson);
    end
  )
  {%- elif is_list_type and is_enum_type -%}
  TJsonHelper.DeserializeList<{{base_type}}>(
    vRoot.GetValue<TJSONArray>({{key}}),
    function (pJson: TJSONValue): T{{prefix}}{{base_type}}
    begin
      Result := T{{prefix}}{{base_type}}.FromString(pJson.Value);
    end
  )
  {%- elif is_list_type -%}
  TJsonHelper.DeserializeList<{{base_type}}>(
    vRoot.GetValue<TJSONArray>({{key}}),
    function (pJson: TJSONValue): {{base_type}}
    begin
      {%if base_type == "integer" -%}
      Result := TJSONNumber(pJson).AsInt;
      {%- elif base_type == "double" -%}
      Result := TJSONNumber(pJson).AsDouble;
      {%- elif base_type == "string" -%}
      Result := TJSONString(pJson).Value;
      {%- elif base_type == "bool" -%}
      Result := TJSONBool(pJson).AsBoolean;
      {%- else -%}
      {{ throw(message= "unsupported type" ~ base_type) }}
      {%- endif %}
    end
  )
  {%- elif is_enum_type -%}
  T{{prefix}}{{base_type}}.FromString(TJsonHelper.TryGetValueOrDefault<TJSONString, String>(vRoot, {{key}}, ''));
  {%- elif base_type == "integer" -%}
  TJsonHelper.TryGetValueOrDefault<TJSONNumber, Integer>(vRoot, {{key}}, 0)
  {%- elif base_type == "double" -%}
  TJsonHelper.TryGetValueOrDefault<TJSONNumber, Double>(vRoot, {{key}}, 0)
  {%- elif base_type == "string" -%}
  TJsonHelper.TryGetValueOrDefault<TJSONString, String>(vRoot, {{key}}, '')
  {%- elif base_type == "bool" -%}
  TJsonHelper.TryGetValueOrDefault<TJSONBool, Boolean>(vRoot, {{key}}, false)
  {%- else -%}
  {{ throw(message= "unsupported type " ~ base_type) }}
  {%- endif -%}
{% endmacro from_json %}

{%- set timestamp = now() | date(format="%d.%m.%Y %H:%m:%S") -%}
// ========================================================================== //
// Generated by Delphi Code Gen - Mode OpenAPI                                //
// {{ self::fixed_size_line(content="Version: " ~ crate_version, size=74) }} //
// {{ self::fixed_size_line(content="Timestamp: " ~ timestamp, size=74) }} //
// ========================================================================== //

unit u{{unitPrefix}}ApiModels;

interface

uses System.Generics.Collections, System.JSON;

type
  {$REGION 'Forward Declerations'}
  {% for classType in classTypes -%}
  T{{prefix}}{{classType.name}} = class;
  {%- endfor %}
  {$ENDREGION}

  {$REGION 'Enums and Helper'}
  {% for enumType in enumTypes -%}
  T{{prefix}}{{enumType.name}} = ({{enumType.variants | map(attribute="name") | join(sep=", ")}});
  {%- endfor %}

  {% for enumType in enumTypes -%}
  T{{prefix}}{{enumType.name}}Helper = record helper for T{{prefix}}{{enumType.name}}
    class function FromString(const pValue: String): T{{prefix}}{{enumType.name}}; static;
  end;
  {%- endfor %}
  {$ENDREGION}

  {$REGION 'Models'}
  {% for classType in classTypes -%}
  T{{prefix}}{{classType.name}} = class
  strict private
    {%- for property in classType.properties %}
    F{{property.name}}: {{ self::type_name(base_type=property.type_name, is_list_type=property.is_list_type, is_reference_type=property.is_reference_type) }};
    {%- endfor -%}{{" "}}
  public
    constructor FromJson(const pJson: String);
    constructor FromJsonRaw(pJson: TJSONValue);
    {% if classType.needs_destructor -%}
    destructor Destroy; override;
    {%- endif -%}
    {{""}}
    {% for property in classType.properties %}
    property {{property.name}}: {{ self::type_name(base_type=property.type_name, is_list_type=property.is_list_type, is_reference_type=property.is_reference_type) }} read F{{property.name}};
    {%- endfor %}
  end;
  {%- endfor %}
  {$ENDREGION}

implementation

uses uJsonHelper,
     System.DateUtils,
     System.SysUtils;

{$REGION 'Enumhelper'}
{% for enumType in enumTypes -%}
class function T{{prefix}}{{enumType.name}}Helper.FromString(const pValue: String): T{{prefix}}{{enumType.name}};
  {% for variant in enumType.variants -%}
  {% if loop.first -%}
  if pValue = '{{variant.key}}' then begin 
    Result := {{variant.name}}
  end
  {%- else -%}
  {{" "}}else if pValue = '{{variant.key}}' then begin 
    Result := {{variant.name}}
  end
  {%- endif -%}
  {%- endfor -%}
  {{" "}}else begin 
    raise Exception.Create('\"' + pValue + '\" is a unknown value for T{{prefix}}{{enumType.name}}');
  end;
end;
{%- endfor %}
{$ENDREGION}

{$REGION 'Models'}
{% for classType in classTypes -%}
{{"{{"}} T{{prefix}}{{classType.name}} {{"}}"}}
const
  {% for property in classType.properties -%}
  cn{{classType.name}}{{property.key}}Key: string = '{{property.key}}';
  {% endfor -%}
{{""}}
constructor T{{prefix}}{{classType.name}}.FromJson(const pJson: String);
begin
  var vRoot := TJSONObject.ParseJSONValue(pJson);

  try
    FromJsonRaw(vRoot);
  finally
    FreeAndNil(vRoot);
  end;
end;

constructor T{{prefix}}{{classType.name}}.FromJsonRaw(pJson: TJSONValue);
begin
  {%- for property in classType.properties %}
  F{{property.name}} := {{ self::from_json(base_type=property.type_name, is_list_type=property.is_list_type, is_reference_type=property.is_reference_type, is_enum_type=property.is_enum_type, key="cn" ~ classType.name ~ property.key ~ "Key") }};
  {%- endfor%}
end;

{% if classType.needs_destructor -%}
destructor T{{prefix}}{{classType.name}}.Destroy;
begin
  {% for property in classType.properties -%}
  {% if property.is_reference_type or property.is_list_type -%}
  FreeAndNil(F{{property.name}});
  {% endif -%}
  {%- endfor %}
  inherited;
end;
{% endif %}
{%- endfor -%}
{$ENDREGION}

end.