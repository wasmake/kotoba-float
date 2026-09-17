use serde::Serialize;
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Latency {
    pub transcript_ms: Option<u64>,
    pub translation_ms: Option<u64>,
}
pub fn safe_error(context: &str, error: impl std::fmt::Display) -> String {
    format!("{context}: {error}")
}
