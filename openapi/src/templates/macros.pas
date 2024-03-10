{% macro fixed_size_line(content, size) %}
  {%- set content_length = content | length -%}
  {%- set space_count = size - content_length -%}
  {{ content }}
  {%- if space_count > 0 -%}
  {%- for v in range(end=space_count) -%}
  {{" "}}
  {%- endfor -%}
  {%- endif -%}
{% endmacro fixed_size_line -%}

{% macro join_args(args) %}
  {%- for arg in args -%}
  p{{arg.name}}: {{arg.type_name}} {%- if not loop.last -%}{{"; "}}{%- endif -%}
  {%- endfor -%}
{% endmacro join_args -%}

{% macro type_name(base_type, is_list_type, is_reference_type, is_enum_type) %}
  {%- if is_list_type and is_reference_type -%}
  TObjectList<T{{prefix}}{{base_type}}>
  {%- elif is_list_type and is_enum_type -%}
  TList<T{{prefix}}{{base_type}}>
  {%- elif is_list_type -%}
  TList<{{base_type}}>
  {%- elif is_reference_type or is_enum_type -%}
  T{{prefix}}{{base_type}}
  {%- elif base_type == "datetime" -%}
  TDateTime
  {%- else -%}
  {{base_type}}
  {%- endif -%}
{% endmacro type_name -%}

{% macro from_json(json_obj_name, base_type, is_list_type, is_reference_type, is_enum_type, key) %}
  {%- if is_list_type and is_reference_type -%}
  TJsonHelper.DeserializeObjectList<T{{prefix}}{{base_type}}>(
    {{json_obj_name}}.GetValue<TJSONArray>({{key}}),
    function (pJson: TJSONValue): T{{prefix}}{{base_type}}
    begin
      Result := T{{prefix}}{{base_type}}.FromJsonRaw(pJson);
    end
  )
  {%- elif is_list_type and is_enum_type -%}
  TJsonHelper.DeserializeList<{{base_type}}>(
    {{json_obj_name}}.GetValue<TJSONArray>({{key}}),
    function (pJson: TJSONValue): T{{prefix}}{{base_type}}
    begin
      Result := T{{prefix}}{{base_type}}.FromString(pJson.Value);
    end
  )
  {%- elif is_list_type -%}
  TJsonHelper.DeserializeList<{{base_type}}>(
    {{json_obj_name}}.GetValue<TJSONArray>({{key}}),
    function (pJson: TJSONValue): {{base_type}}
    begin
      {%if base_type == "integer" -%}
      Result := TJSONNumber(pJson).AsInt;
      {%- elif base_type == "double" -%}
      Result := TJSONNumber(pJson).AsDouble;
      {%- elif base_type == "string" -%}
      Result := TJSONString(pJson).Value;
      {%- elif base_type == "boolean" -%}
      Result := TJSONBool(pJson).AsBoolean;
      {%- elif base_type == "datetime" -%}
      Result := ISO8601ToDate(TJSONString(pJson).Value);
      {%- else -%}
      {{ throw(message= "unsupported type " ~ base_type) }}
      {%- endif %}
    end
  )
  {%- elif is_enum_type -%}
  T{{prefix}}{{base_type}}.FromString(TJsonHelper.TryGetValueOrDefault<TJSONString, String>({{json_obj_name}}, {{key}}, ''))
  {%- elif is_reference_type -%}
  {{ self::type_name(base_type=base_type, is_list_type=is_list_type, is_reference_type=is_reference_type, is_enum_type=is_enum_type) }}.FromJsonRaw({{json_obj_name}}.GetValue<TJSONObject>({{key}}))
  {%- elif base_type == "integer" -%}
  TJsonHelper.TryGetValueOrDefault<TJSONNumber, Integer>({{json_obj_name}}, {{key}}, 0)
  {%- elif base_type == "double" -%}
  TJsonHelper.TryGetValueOrDefault<TJSONNumber, Double>({{json_obj_name}}, {{key}}, 0)
  {%- elif base_type == "string" -%}
  TJsonHelper.TryGetValueOrDefault<TJSONString, String>({{json_obj_name}}, {{key}}, '')
  {%- elif base_type == "boolean" -%}
  TJsonHelper.TryGetValueOrDefault<TJSONBool, Boolean>({{json_obj_name}}, {{key}}, false)
  {%- elif base_type == "datetime" -%}
  ISO8601ToDate(TJsonHelper.TryGetValueOrDefault<TJSONString, String>({{json_obj_name}}, {{key}}, ''))
  {%- else -%}
  {{ throw(message= "unsupported type " ~ base_type) }}
  {%- endif -%}
{% endmacro from_json -%}

{% macro from_json_raw(json_obj_name, base_type, is_list_type, is_reference_type, is_enum_type) %}
  {%- if is_reference_type -%}
  T{{prefix}}{{base_type}}.FromJsonRaw({{json_obj_name}})
  {%- elif is_enum_type -%}
  T{{prefix}}{{base_type}}.FromString({{json_obj_name}}.Value)
  {% elif base_type == "integer" -%}
  TJSONNumber({{json_obj_name}}).AsInt
  {%- elif base_type == "double" -%}
  TJSONNumber({{json_obj_name}}).AsDouble
  {%- elif base_type == "string" -%}
  TJSONString({{json_obj_name}}).Value
  {%- elif base_type == "boolean" -%}
  TJSONBool({{json_obj_name}}).AsBoolean
  {%- elif base_type == "datetime" -%}
  ISO8601ToDate(TJSONString({{json_obj_name}}).Value)
  {%- else -%}
  {{ throw(message= "unsupported type " ~ base_type) }}
  {%- endif -%}
{% endmacro from_json_raw -%}