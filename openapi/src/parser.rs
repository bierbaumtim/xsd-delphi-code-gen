use types::*;

pub mod types;

/// Deserialize an OpenAPI document from a JSON string.
pub fn from_json_str(s: &str) -> anyhow::Result<OpenAPI> {
    let doc: OpenAPI = serde_json::from_str(s)?;
    Ok(doc)
}

pub struct Resolver<'a> {
    api: &'a OpenAPI,
}

impl<'a> Resolver<'a> {
    pub fn new(api: &'a OpenAPI) -> Self {
        Resolver { api }
    }

    /// Resolve a `SchemaOrRef` to a `&Schema`.
    pub fn resolve_schema(&self, sr: &'a SchemaOrRef) -> Option<&'a Schema> {
        match sr {
            SchemaOrRef::Item(schema) => Some(schema),
            SchemaOrRef::Ref { reference } => self.resolve_schema_ref(reference),
        }
    }

    /// Resolve a `ParameterOrRef` to a `&Parameter`.
    pub fn resolve_parameter(&self, pr: &'a ParameterOrRef) -> Option<&'a Parameter> {
        match pr {
            ParameterOrRef::Item(param) => Some(param),
            ParameterOrRef::Ref { reference } => self.resolve_parameter_ref(reference),
        }
    }

    /// Resolve a `ResponseOrRef` to a `&Response`.
    pub fn resolve_response(&self, rr: &'a ResponseOrRef) -> Option<&'a Response> {
        match rr {
            ResponseOrRef::Item(resp) => Some(resp),
            ResponseOrRef::Ref { reference } => self.resolve_response_ref(reference),
        }
    }

    /// Resolve a `RequestBodyOrRef` to a `&RequestBody`.
    pub fn resolve_request_body(&self, rr: &'a RequestBodyOrRef) -> Option<&'a RequestBody> {
        match rr {
            RequestBodyOrRef::Item(rb) => Some(rb),
            RequestBodyOrRef::Ref { reference } => self.resolve_request_body_ref(reference),
        }
    }

    // … you can add the other OrRef types (HeaderOrRef, ExampleOrRef, etc.) here …

    // Internal helpers for the standard `#/components/...` paths:

    fn resolve_schema_ref(&self, reference: &str) -> Option<&'a Schema> {
        self.resolve(
            reference,
            |comp| comp.schemas.get(comp_key(reference, "schemas/")),
            |or_ref| match or_ref {
                SchemaOrRef::Item(x) => Some(x),
                _ => None,
            },
        )
    }

    fn resolve_parameter_ref(&self, reference: &str) -> Option<&'a Parameter> {
        self.resolve(
            reference,
            |comp| comp.parameters.get(comp_key(reference, "parameters/")),
            |or_ref| match or_ref {
                ParameterOrRef::Item(x) => Some(x),
                _ => None,
            },
        )
    }

    fn resolve_response_ref(&self, reference: &str) -> Option<&'a Response> {
        self.resolve(
            reference,
            |comp| comp.responses.get(comp_key(reference, "responses/")),
            |or_ref| match or_ref {
                ResponseOrRef::Item(x) => Some(x),
                _ => None,
            },
        )
    }

    fn resolve_request_body_ref(&self, reference: &str) -> Option<&'a RequestBody> {
        self.resolve(
            reference,
            |comp| {
                comp.request_bodies
                    .get(comp_key(reference, "requestBodies/"))
            },
            |or_ref| match or_ref {
                RequestBodyOrRef::Item(x) => Some(x),
                _ => None,
            },
        )
    }

    /// A generic helper for any `XxxOrRef`.
    fn resolve<OrRef, Item, F, G>(
        &self,
        reference: &str,
        get_from_comp: F,
        extract: G,
    ) -> Option<&'a Item>
    where
        OrRef: 'a,
        F: FnOnce(&'a Components) -> Option<&'a OrRef>,
        G: FnOnce(&'a OrRef) -> Option<&'a Item>,
    {
        // must start with "#/components/"
        const PREFIX: &str = "#/components/";
        if !reference.starts_with(PREFIX) {
            return None;
        }
        let comps = self.api.components.as_ref()?;
        // extract the sub‐path after "#/components/"
        let tail = &reference[PREFIX.len()..];
        // get the OrRef out of the right map
        let or_ref = get_from_comp(comps)?;
        extract(or_ref)
    }
}

/// Helper: strip off "#/components/<section>/" and return just the key.
fn comp_key<'a>(reference: &'a str, section: &str) -> &'a str {
    reference
        .strip_prefix(&format!("#/components/{}", section))
        .unwrap_or_default()
}
