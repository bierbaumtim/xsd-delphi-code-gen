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

{% macro class_declaration(class) -%}
  // XML Qualified Name: {{class.qualified_name}}
  {% for line in class.documentations -%}
  // {{line}}
  {% endfor -%}
  {{class.name}} = class({{class.super_type | default(value="TObject") }})
  {%- if class.has_optional_fields %}
  strict private
    {% for variable in class.optional_variables -%}
    F{{variable.name}}: TOptional<{{variable.data_type_repr}}>;
    {% endfor -%}
    {{""}}
    {% for variable in class.optional_variables -%}
    procedure Set{{variable.name}}(pValue: TOptional<{{variable.data_type_repr}}>);
    {% endfor -%}
  {%- endif %}
  public
    {% if has_constant_fields -%}
      {% for variable in class.constant_variables -%}
      const {{variable.name}}: {{variable.data_type_repr}} = {{variable.default_value}};
      {% endfor -%}
      var
    {% endif -%}
    {% if class.variables | length > 0 -%}
    {% for variable in class.variables -%}
    {% if variable.required -%}
    /// <summary>Required</summary>
    {% endif -%}
    {% for line in variable.documentations -%}
    // {{line}}
    {% endfor -%}
    {{variable.name}}: {{variable.data_type_repr}};
    {% endfor %}
    {% endif -%}
    {% if gen_to_xml -%}
    constructor Create; {% if class.super_type %}override;{% else %}virtual;{% endif %}
    {% endif -%}
    {% if gen_from_xml -%}
    constructor FromXml(node: IXMLNode); {% if class.super_type %}override;{% else %}virtual;{% endif %}
    {% endif -%}
    {% if class.needs_destructor -%}
    destructor Destroy; override;
    {% endif -%}
    {{""}}
    {% if gen_to_xml -%}
    procedure AppendToXmlRaw(pParent: IXMLNode); {% if class.super_type %}override;{% else %}virtual;{% endif %}
    function ToXml: String; {% if class.super_type %}override;{% else %}virtual;{% endif %}
    {%- endif %}
    {%- if class.has_optional_fields %}
    {% for variable in class.optional_variables %}
    {%- for line in variable.documentations %}
    // {{line}}
    {%- endfor %}
    property {{variable.name}}: TOptional<{{variable.data_type_repr}}> read F{{variable.name}} write Set{{variable.name}};
    {%- endfor %}
    {%- endif %}
  end;
{%- endmacro class_declaration -%}

{% macro document_class_declaration(class) -%}
  // XML Qualified Name: {{class.qualified_name}}
  {% for line in class.documentations -%}
  // {{line}}
  {% endfor -%}
  {{class.name}} = class({{class.super_type | default(value="TObject") }})
  {%- if class.has_optional_fields %}
  strict private
    {% for variable in class.optional_variables -%}
    F{{variable.name}}: TOptional<{{variable.data_type_repr}}>;
    {% endfor -%}
    {{""}}
    {% for variable in class.optional_variables -%}
    procedure Set{{variable.name}}(pValue: TOptional<{{variable.data_type_repr}}>);
    {% endfor -%}
  {%- endif %}
  public
    {% if has_constant_fields -%}
      {% for variable in class.constant_variables -%}
      const {{variable.name}}: {{variable.data_type_repr}} = {{variable.default_value}};
      {% endfor -%}
      var
    {% endif -%}
    {% if class.variables | length > 0 -%}
    {% for variable in class.variables -%}
    {% if variable.required -%}
    /// <summary>Required</summary>
    {% endif -%}
    {% for line in variable.documentations -%}
    // {{line}}
    {% endfor -%}
    {{variable.name}}: {{variable.data_type_repr}};
    {% endfor %}
    {% endif -%}
    {% if gen_to_xml -%}
    constructor Create; {% if class.super_type %}override;{% else %}virtual;{% endif %}
    {% endif -%}
    {% if gen_from_xml -%}
    constructor FromXml(node: IXMLNode); {% if class.super_type %}override;{% else %}virtual;{% endif %}
    {% endif -%}
    {% if class.needs_destructor -%}
    destructor Destroy; override;
    {% endif -%}
    {{""}}
    {% if gen_to_xml -%}
    procedure AppendToXmlRaw(pParent: IXMLNode); {% if class.super_type %}override;{% else %}virtual;{% endif %}
    function ToXml: String; {% if class.super_type %}override;{% else %}virtual;{% endif %}
    /// <summary>Validates the XML instance against the XSD schema</summary>
    /// <returns>True if validation succeeds, False otherwise</returns>
    function Validate(out pErrorMsg: String): Boolean; virtual;
    {%- endif %}
    {%- if class.has_optional_fields %}
    {% for variable in class.optional_variables %}
    {%- for line in variable.documentations %}
    // {{line}}
    {%- endfor %}
    property {{variable.name}}: TOptional<{{variable.data_type_repr}}> read F{{variable.name}} write Set{{variable.name}};
    {%- endfor %}
    {%- endif %}
  end;
{%- endmacro document_class_declaration -%}

{% macro class_implementation(class) -%}
{{"{"}} {{class.name}} {{"}"}}
{% if gen_to_xml -%}
constructor {{class.name}}.Create;
begin
  {%- if class.super_type %}
  inherited;
  {% endif %}
  {%- for initializer in class.variable_initializer %}
  {{initializer}}
  {%- endfor %}
end;
{%- endif %}

{% if gen_from_xml -%}
constructor {{class.name}}.FromXml(node: IXMLNode);
begin
  {%- if class.super_type %}
  inherited;
  {%- endif %}

  {%- if class.deserialize_element_variables | length > 0 %}
  // Variables
  {%- if class.has_optional_element_variables %}
  var vOptionalNode: IXMLNode;
  {%- endif %}
  {% for element in class.deserialize_element_variables %}
  {%- if element.is_list %}
  {{element.name}} := {{element.data_type_repr}}.Create;

  var __{{element.name}}Index := node.ChildNodes.IndexOf('{{element.xml_name}}');
  if __{{element.name}}Index >= 0 then begin
    for var I := 0 to node.ChildNodes.Count - __{{element.name}}Index - 1 do begin
      var __{{element.name}}Node := node.ChildNodes[__{{element.name}}Index + I];

      if __{{element.name}}Node.LocalName <> '{{element.xml_name}}' then continue;

      {{element.name}}.Add({{element.from_xml_code}});
    end;
  end;
  {% elif element.is_inline_list %}
  {{element.name}} := {{element.data_type_repr}}.Create;

  {%- if element.is_required %}
  for var vPart in node.ChildNodes['{{element.xml_name}}'].Text.Split([' ']) do begin
    {{element.name}}.Add({{element.from_xml_code}});
  end;
  {% else %}
  vOptionalNode := node.ChildNodes.FindNode('{{element.xml_name}}');
  if Assigned(vOptionalNode) then begin
    for var vPart in vOptionalNode.Text.Split([' ']) do begin
      {{element.name}}.Add({{element.from_xml_code}});
    end;
  end;
  {% endif %}
  {%- elif element.is_fixed_size_list %}
  {% for i in range(end=element.fixed_size_list_size) %}
  {{element.name}}{{ i + 1 }} := Default({{element.data_type_repr}});
  {%- endfor %}

  var __{{element.name}}Index := node.ChildNodes.IndexOf('{{element.xml_name}}');
  if __{{element.name}}Index >= 0 then begin
    for var I := 0 to {{element.fixed_size_list_size - 1}} do begin
      var __{{element.name}}Node := node.ChildNodes[__{{element.name}}Index + I];

      if __{{element.name}}Node.LocalName <> '{{element.xml_name}}' then break;

      case I of
      {%- for i in range(end=element.fixed_size_list_size) %}
        {{i}}: {{element.name}}{{ i + 1 }} := {{element.from_xml_code}};
      {%- endfor %}
      end;
    end;
  end;
  {% elif element.is_required %}
  {{element.name}} := {{element.from_xml_code}};
  {%- elif element.has_optional_wrapper %}
  vOptionalNode := node.ChildNodes.FindNode('{{element.xml_name}}');
  if Assigned(vOptionalNode) then begin
    F{{element.name}} := TSome<{{element.data_type_repr}}>.Create({{element.from_xml_code}});
  end else begin
    F{{element.name}} := TNone<{{element.data_type_repr}}>.Create;
  end;
  {% else %}
  vOptionalNode := node.ChildNodes.FindNode('{{element.xml_name}}');
  if Assigned(vOptionalNode) then begin
    {{element.name}} := {{element.from_xml_code}};
  end else begin
    {{element.name}} := nil;
  end;
  {% endif %}
  {%- endfor %}
  {%- endif %}

  {%- if class.deserialize_attribute_variables | length > 0 %}
  // Attributes
  {%- for attr in deserialize_attribute_variables %}
  if node.HasAttribute('{{attr.xml_value}}') then begin
    {% if attr.has_optional_wrapper %}F{% endif %}{{attr.name}} := {{attr.from_xml_code_available}};
  end else begin
    {% if attr.has_optional_wrapper %}F{% endif %}{{attr.name}} := {{attr.from_xml_code_missing}};
  end;
  {%- endfor %}
  {%- endif %}
end;
{%- endif %}

{% if gen_to_xml -%}
procedure {{class.name}}.AppendToXmlRaw(pParent: IXMLNode);
begin
  {%- if class.super_type %}
  inherited;
  {% endif %}
  var node: IXMLNode;
{% for variable in class.serialize_variables -%}
{%- if variables.is_list %}
  for var __Item in {{variable.name}} do begin
  {%- if variable.is_class %}
    node := pParent.AddChild('{{variable.xml_name}}');
    __Item.AppendToXmlRaw(node);
  {%- elif variable.is_enum %}
    node := pParent.AddChild('{{variable.xml_name}}');
    node.Text := __Item.ToXmlValue;
  {%- else %}
    node := pParent.AddChild('{{variable.xml_name}}');
    node.Text := {{variable.to_xml_code}};
  {%- endif %}
  end;
{%- elif variable.is_inline_list %}
  {%- if variable.is_required %}
  node := pParent.AddChild('{{variable.xml_name}}');
  for var I := 0 to {{variable.name}}.Count - 1 do begin
    node.Text := node.Text + {{variable.to_xml_code}};

    if I < {{variable.name}}.Count - 1 then begin
      node.Text := node.Text + ' ';
    end;
  end;
  {%- else %}
  if Assigned({{variable.name}}) then begin
    node := pParent.AddChild('{{variable.xml_name}}');
    for var I := 0 to {{variable.name}}.Count - 1 do begin
      node.Text := node.Text + {{variable.to_xml_code}};

      if I < {{variable.name}}.Count - 1 then begin
        node.Text := node.Text + ' ';
      end;
    end;
  end;
  {%- endif %}
{%- elif variable.is_class %}
  {%- if variable.is_required %}
  node := pParent.AddChild('{{variable.xml_name}}');
  {{variable.name}}.AppendToXmlRaw(node);
  {%- else %}
  if Assigned({{variable.name}}) then begin
    node := pParent.AddChild('{{variable.xml_name}}');
    {{variable.name}}.AppendToXmlRaw(node);
  end;
  {%- endif %}
{%- elif variable.is_enum %}
  {% if variable.has_optional_wrapper %}
  if F{{variable.name}}.IsSome then begin
    node := pParent.AddChild('{{variable.xml_name}}');
    node.Text := F{{variable.name}}.Unwrap.ToXmlValue;
  end;
  {%- else %}
  node := pParent.AddChild('{{variable.xml_name}}');
  node.Text := {{variable.name}}.ToXmlValue;
  {%- endif %}
{%- elif variable.has_optional_wrapper %}
  if F{{variable.name}}.IsSome then begin
    node := pParent.AddChild('{{variable.xml_name}}');
    node.Text := {{variable.to_xml_code}};
  end;
{%- else %}
  node := pParent.AddChild('{{variable.xml_name}}');
  node.Text := {{variable.to_xml_code}};
{% endif %}
{%- endfor %}
end;

function {{class.name}}.ToXml: String;
begin
  var vXmlDoc := NewXMLDocument;

  AppendToXmlRaw(vXmlDoc.Node);

  vXmlDoc.SaveToXML(Result);
end;
{% endif -%}
{% if class.optional_variables | length > 0 -%}
{% for variable in class.optional_variables %}
procedure {{class.name}}.Set{{variable.name}}(pValue: TOptional<{{variable.data_type_repr}}>);
begin
  if F{{variable.name}} <> pValue then F{{variable.name}}.Free;

  if (not Assigned(pValue)) or (pValue = nil) then begin
    F{{variable.name}} := TNone<{{variable.data_type_repr}}>.Create;
  end else begin
    F{{variable.name}} := pValue;
  end;
end;
{% endfor -%}
{%- endif %}

{% if class.needs_destructor -%}
destructor {{class.name}}.Destroy;
begin
  {%- for variable in class.variables | filter(attribute="requires_free", value=true) %}
  {{variable.name}}.Free;
  {%- endfor %}
  {%- for variable in class.optional_variables %}
  F{{variable.name}}.Free;
  {%- endfor %}

  inherited;
end;
{%- endif %}
{%- endmacro class_implementation -%}

{% macro document_class_validation_implementation(class) -%}
function {{class.name}}.Validate(out pErrorMsg: String): Boolean;
var
  vXmlDoc: IXMLDocument;
  vSchemaCache: IXMLDOMSchemaCollection2;
  vDomDoc: IXMLDOMDocument2;
  vParseError: IXMLDOMParseError;
begin
  Result := False;
  pErrorMsg := '';

  try
    // Create COM objects for validation
    vSchemaCache := CreateComObject(CLASS_XMLSchemaCache60) as IXMLDOMSchemaCollection2;
    vDomDoc := CreateComObject(CLASS_DOMDocument60) as IXMLDOMDocument2;

    // Load schemas from uValidationSchemes unit
    uValidationSchemes.LoadSchemas(vSchemaCache);

    // Configure DOM document for validation
    vDomDoc.async := False;
    vDomDoc.validateOnParse := True;
    vDomDoc.resolveExternals := False;
    vDomDoc.schemas := vSchemaCache;

    // Load XML to validate
    vDomDoc.loadXML(Self.ToXml);

    // Check for errors
    vParseError := vDomDoc.validate;
    if vParseError.errorCode <> 0 then begin
      pErrorMsg := Format('Validation failed at line %d, position %d: %s',
        [vParseError.line, vParseError.linepos, vParseError.reason]);
      Exit(False);
    end;

    Result := True;
  except
    on E: Exception do begin
      pErrorMsg := 'Validation error: ' + E.Message;
      Result := False;
    end;
  end;
end;
{%- endmacro document_class_validation_implementation -%}