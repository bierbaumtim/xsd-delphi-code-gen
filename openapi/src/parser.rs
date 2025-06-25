use types::*;

pub mod types;

/// Deserialize an OpenAPI document from a JSON string.
pub fn from_json_str(s: &str) -> anyhow::Result<OpenAPI> {
    let doc: OpenAPI = serde_json::from_str(s)?;
    Ok(doc)
}
