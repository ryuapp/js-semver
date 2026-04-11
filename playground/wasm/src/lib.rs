use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use js_semver::{Range, Version};

#[derive(Serialize)]
struct ParseResult {
    input: String,
    ok: bool,
    canonical: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct SatisfiesResult {
    range: ParseResult,
    version: ParseResult,
    satisfies: Option<bool>,
}

fn serialize<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| {
        String::from(
            r#"{"ok":false,"canonical":null,"error":"serialization failure"}"#,
        )
    })
}

fn parse_range_result(input: &str) -> ParseResult {
    match Range::parse(input) {
        Ok(range) => ParseResult {
            input: input.to_owned(),
            ok: true,
            canonical: Some(range.to_string()),
            error: None,
        },
        Err(error) => ParseResult {
            input: input.to_owned(),
            ok: false,
            canonical: None,
            error: Some(error.to_string()),
        },
    }
}

fn parse_version_result(input: &str) -> ParseResult {
    match Version::parse(input) {
        Ok(version) => ParseResult {
            input: input.to_owned(),
            ok: true,
            canonical: Some(version.to_string()),
            error: None,
        },
        Err(error) => ParseResult {
            input: input.to_owned(),
            ok: false,
            canonical: None,
            error: Some(error.to_string()),
        },
    }
}

#[wasm_bindgen]
pub fn parse_range(input: &str) -> String {
    serialize(&parse_range_result(input))
}

#[wasm_bindgen]
pub fn parse_version(input: &str) -> String {
    serialize(&parse_version_result(input))
}

#[wasm_bindgen]
pub fn satisfies(range_input: &str, version_input: &str) -> String {
    let range_result = parse_range_result(range_input);
    let version_result = parse_version_result(version_input);

    let satisfies = match (Range::parse(range_input), Version::parse(version_input)) {
        (Ok(range), Ok(version)) => Some(range.satisfies(&version)),
        _ => None,
    };

    serialize(&SatisfiesResult {
        range: range_result,
        version: version_result,
        satisfies,
    })
}
