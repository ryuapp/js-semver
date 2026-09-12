use wasm_bindgen::prelude::{wasm_bindgen, JsError};

use js_semver::{Range, Version};

fn parse_range_inner(input: &str) -> Result<Range, JsError> {
    Range::parse(input).map_err(|error| JsError::new(&error.to_string()))
}

fn parse_version_inner(input: &str) -> Result<Version, JsError> {
    Version::parse(input).map_err(|error| JsError::new(&error.to_string()))
}

#[wasm_bindgen]
pub fn parse_range(input: &str) -> Result<String, JsError> {
    let range = parse_range_inner(input)?;
    Ok(range.to_string())
}

#[wasm_bindgen]
pub fn parse_version(input: &str) -> Result<String, JsError> {
    let version = parse_version_inner(input)?;
    Ok(version.to_string())
}

#[wasm_bindgen]
pub fn satisfies(range_input: &str, version_input: &str) -> Option<bool> {
    let range = parse_range_inner(range_input).ok()?;
    let version = parse_version_inner(version_input).ok()?;
    Some(range.satisfies(&version))
}
