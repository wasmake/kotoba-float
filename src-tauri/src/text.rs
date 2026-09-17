use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct TextResult {
    pub romaji: String,
    pub translation: String,
}
pub fn validate(raw: &str) -> Result<TextResult, String> {
    let value: TextResult =
        serde_json::from_str(raw).map_err(|_| "Malformed translation response".to_string())?;
    if value.romaji.trim().is_empty() || value.translation.trim().is_empty() {
        return Err("Empty translation response".into());
    }
    Ok(value)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_response_validation() {
        assert!(validate(r#"{"romaji":"ohayou","translation":"Buenos días"}"#).is_ok());
        assert!(validate(r#"{"translation":"x"}"#).is_err());
        assert!(validate("not json").is_err());
    }
}
