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
  p{{arg.name}}: {{arg.type_name}} {%- if not loop.last -%}{{", "}}{%- endif -%}
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