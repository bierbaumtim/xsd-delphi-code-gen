//! The data model for OpenAPI 3.1 (all structs matching the spec).

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Top-level OpenAPI document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAPI {
    pub openapi: String, // MUST be "3.1.0"
    pub info: Info,
    #[serde(default)]
    pub servers: Vec<Server>,
    pub paths: Paths,
    #[serde(default)]
    pub components: Option<Components>,
    #[serde(default)]
    pub security: Vec<SecurityRequirement>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub external_docs: Option<ExternalDocumentation>,

    /// Any `x-` extensions
    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

/// General API metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub terms_of_service: Option<String>,
    #[serde(default)]
    pub contact: Option<Contact>,
    #[serde(default)]
    pub license: Option<License>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contact {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub email: Option<String>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Server {
    pub url: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub variables: IndexMap<String, ServerVariable>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerVariable {
    #[serde(rename = "enum", default)]
    pub enum_values: Vec<String>,
    pub default: String,
    #[serde(default)]
    pub description: Option<String>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

/// A map of path template → [`PathItem`].
pub type Paths = IndexMap<String, PathItem>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathItem {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub get: Option<Operation>,
    #[serde(default)]
    pub put: Option<Operation>,
    #[serde(default)]
    pub post: Option<Operation>,
    #[serde(default)]
    pub delete: Option<Operation>,
    #[serde(default)]
    pub options: Option<Operation>,
    #[serde(default)]
    pub head: Option<Operation>,
    #[serde(default)]
    pub patch: Option<Operation>,
    #[serde(default)]
    pub trace: Option<Operation>,

    #[serde(default)]
    pub servers: Vec<Server>,
    #[serde(default)]
    pub parameters: Vec<ParameterOrRef>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub external_docs: Option<ExternalDocumentation>,
    #[serde(default)]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub parameters: Vec<ParameterOrRef>,
    #[serde(default)]
    pub request_body: Option<RequestBodyOrRef>,
    #[serde(default)]
    pub responses: Responses,
    #[serde(default)]
    pub callbacks: IndexMap<String, CallbackOrRef>,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub security: Vec<SecurityRequirement>,
    #[serde(default)]
    pub servers: Vec<Server>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

pub type Responses = IndexMap<String, ResponseOrRef>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseOrRef {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Item(Response),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    #[serde(default)]
    pub headers: IndexMap<String, HeaderOrRef>,
    #[serde(default)]
    pub content: IndexMap<String, MediaType>,
    #[serde(default)]
    pub links: IndexMap<String, LinkOrRef>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaType {
    #[serde(default)]
    pub schema: Option<SchemaOrRef>,
    #[serde(default)]
    pub example: Option<Value>,
    #[serde(default)]
    pub examples: IndexMap<String, ExampleOrRef>,
    #[serde(default)]
    pub encoding: IndexMap<String, Encoding>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

/// Components object, holds reusable definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Components {
    #[serde(default)]
    pub schemas: IndexMap<String, SchemaOrRef>,
    #[serde(default)]
    pub responses: IndexMap<String, ResponseOrRef>,
    #[serde(default)]
    pub parameters: IndexMap<String, ParameterOrRef>,
    #[serde(default)]
    pub examples: IndexMap<String, ExampleOrRef>,
    #[serde(default)]
    pub request_bodies: IndexMap<String, RequestBodyOrRef>,
    #[serde(default)]
    pub headers: IndexMap<String, HeaderOrRef>,
    #[serde(default)]
    pub security_schemes: IndexMap<String, SecuritySchemeOrRef>,
    #[serde(default)]
    pub links: IndexMap<String, LinkOrRef>,
    #[serde(default)]
    pub callbacks: IndexMap<String, CallbackOrRef>,
    #[serde(default)]
    pub path_items: IndexMap<String, PathItemOrRef>,
    #[serde(default)]
    pub webhooks: IndexMap<String, PathItemOrRef>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SchemaOrRef {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Item(Schema),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub multiple_of: Option<f64>,
    #[serde(default)]
    pub maximum: Option<f64>,
    #[serde(default)]
    pub exclusive_maximum: bool,
    #[serde(default)]
    pub minimum: Option<f64>,
    #[serde(default)]
    pub exclusive_minimum: bool,
    #[serde(default)]
    pub max_length: Option<u64>,
    #[serde(default)]
    pub min_length: Option<u64>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub max_items: Option<u64>,
    #[serde(default)]
    pub min_items: Option<u64>,
    #[serde(default)]
    pub unique_items: bool,
    #[serde(default)]
    pub max_properties: Option<u64>,
    #[serde(default)]
    pub min_properties: Option<u64>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub discriminator: Option<Discriminator>,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub write_only: bool,
    #[serde(default)]
    pub xml: Option<XML>,
    #[serde(default)]
    pub external_docs: Option<ExternalDocumentation>,
    #[serde(default)]
    pub example: Option<Value>,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    #[serde(rename = "enum")]
    pub enum_: Vec<Value>, // `enum` is a reserved keyword in Rust, so we use `enum_`

    // Composition
    #[serde(default)]
    pub all_of: Vec<SchemaOrRef>,
    #[serde(default)]
    pub one_of: Vec<SchemaOrRef>,
    #[serde(default)]
    pub any_of: Vec<SchemaOrRef>,
    #[serde(default)]
    pub not: Option<Box<SchemaOrRef>>,

    // Object
    #[serde(default)]
    pub properties: IndexMap<String, SchemaOrRef>,
    #[serde(default)]
    pub additional_properties: Option<Box<SchemaOrRefOrBool>>,

    // Array
    #[serde(default)]
    pub items: Option<Box<SchemaOrRef>>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SchemaOrRefOrBool {
    Ref(SchemaOrRef),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Discriminator {
    pub property_name: String,
    #[serde(default)]
    pub mapping: IndexMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XML {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub attribute: bool,
    #[serde(default)]
    pub wrapped: bool,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParameterOrRef {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Item(Parameter),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub in_: String,
    #[serde(default)]
    #[serde(rename = "in")]
    pub _in: (),
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub allow_empty_value: bool,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub explode: Option<bool>,
    #[serde(default)]
    pub allow_reserved: bool,

    #[serde(default)]
    pub schema: Option<SchemaOrRef>,
    #[serde(default)]
    pub content: IndexMap<String, MediaType>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestBodyOrRef {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Item(RequestBody),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    #[serde(default)]
    pub description: Option<String>,
    pub content: IndexMap<String, MediaType>,
    #[serde(default)]
    pub required: bool,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HeaderOrRef {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Item(Header),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub allow_empty_value: bool,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub explode: Option<bool>,
    #[serde(default)]
    pub allow_reserved: bool,

    #[serde(default)]
    pub schema: Option<SchemaOrRef>,
    #[serde(default)]
    pub content: IndexMap<String, MediaType>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LinkOrRef {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Item(Link),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    #[serde(default)]
    pub operation_ref: Option<String>,
    #[serde(default)]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub parameters: IndexMap<String, Value>,
    #[serde(default)]
    pub request_body: Option<Value>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub server: Option<Server>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExampleOrRef {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Item(Example),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Example {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub external_value: Option<String>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CallbackOrRef {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Item(Callback),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Callback(pub IndexMap<String, PathItem>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityRequirement(pub IndexMap<String, Vec<String>>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SecuritySchemeOrRef {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Item(SecurityScheme),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub in_: Option<String>,
    #[serde(default)]
    pub scheme: Option<String>,
    #[serde(default)]
    pub bearer_format: Option<String>,
    #[serde(default)]
    pub flows: Option<OAuthFlows>,
    #[serde(default)]
    pub open_id_connect_url: Option<String>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthFlows {
    #[serde(default)]
    pub implicit: Option<OAuthFlow>,
    #[serde(default)]
    pub password: Option<OAuthFlow>,
    #[serde(default)]
    pub client_credentials: Option<OAuthFlow>,
    #[serde(default)]
    pub authorization_code: Option<OAuthFlow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthFlow {
    pub authorization_url: String,
    pub token_url: String,
    #[serde(default)]
    pub refresh_url: Option<String>,
    pub scopes: IndexMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub external_docs: Option<ExternalDocumentation>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDocumentation {
    pub url: String,
    #[serde(default)]
    pub description: Option<String>,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Encoding {
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub headers: IndexMap<String, HeaderOrRef>,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub explode: Option<bool>,
    #[serde(default)]
    pub allow_reserved: bool,

    #[serde(flatten)]
    pub extensions: IndexMap<String, Value>,
}

/// A `$ref`able PathItem in Components/webhooks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathItemOrRef {
    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Item(PathItem),
}

impl OpenAPI {
    pub fn tags_with_sort_id(&self) -> Vec<(usize, &Tag)> {
        self.tags
            .iter()
            .enumerate()
            .map(|(sortid, tag)| (sortid, tag))
            .collect::<Vec<_>>()
    }

    pub fn resolve_schema(&self, reference: &str) -> Option<&Schema> {
        self.components
            .as_ref()?
            .schemas
            .get(reference.split("/").last().unwrap_or(reference))
            .and_then(|sr| match sr {
                SchemaOrRef::Item(schema) => Some(schema),
                SchemaOrRef::Ref { reference } => self.resolve_schema(reference),
            })
    }

    pub fn resolve_parameter(&self, reference: &str) -> Option<&Parameter> {
        self.components
            .as_ref()?
            .parameters
            .get(reference)
            .and_then(|pr| match pr {
                ParameterOrRef::Item(param) => Some(param),
                ParameterOrRef::Ref { reference } => self.resolve_parameter(reference),
            })
    }

    pub fn resolve_response(&self, reference: &str) -> Option<&Response> {
        self.components
            .as_ref()?
            .responses
            .get(reference)
            .and_then(|rr| match rr {
                ResponseOrRef::Item(resp) => Some(resp),
                ResponseOrRef::Ref { reference } => self.resolve_response(reference),
            })
    }

    pub fn resolve_request_body(&self, reference: &str) -> Option<&RequestBody> {
        self.components
            .as_ref()?
            .request_bodies
            .get(reference)
            .and_then(|rr| match rr {
                RequestBodyOrRef::Item(rb) => Some(rb),
                RequestBodyOrRef::Ref { reference } => self.resolve_request_body(reference),
            })
    }
}
