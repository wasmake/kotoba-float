use serde::Serialize;

pub const BLOCK_REASON: &str = "ChatGPT subscription OAuth is not available to this third-party app. API-key mode uses the separately billed OpenAI API.";
const SERVICE: &str = "app.kotobafloat.desktop";
const USER: &str = "openai-api-key";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub identity: bool,
    pub transcription: bool,
    pub translation: bool,
    pub reason: String,
    pub checked_at: String,
}

pub fn capabilities(verified: u8) -> Capabilities {
    let connected = load_api_key().is_ok();
    Capabilities {
        identity: connected,
        transcription: verified & 1 != 0,
        translation: verified & 2 != 0,
        reason: if connected { "API key stored in the operating-system credential store. API usage is billed separately from ChatGPT." } else { BLOCK_REASON }.into(),
        checked_at: chrono::Utc::now().to_rfc3339(),
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, USER).map_err(|e| format!("Credential store unavailable: {e}"))
}
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn save_api_key(value: &str) -> Result<(), String> {
    let value = value.trim();
    if !value.starts_with("sk-") || value.len() < 20 {
        return Err("Enter a valid OpenAI API key".into());
    }
    entry()?
        .set_password(value)
        .map_err(|e| format!("Could not save credential: {e}"))
}
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn load_api_key() -> Result<String, String> {
    entry()?
        .get_password()
        .map_err(|_| "No OpenAI API key is connected".into())
}
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn clear_api_key() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Could not clear credential: {e}")),
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn save_api_key(_: &str) -> Result<(), String> {
    Err("API credentials are supported only on Windows and macOS builds".into())
}
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn load_api_key() -> Result<String, String> {
    Err("No OpenAI API key is connected".into())
}
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn clear_api_key() -> Result<(), String> {
    Ok(())
}

pub trait CredentialStore {
    fn put(&mut self, value: &str);
    fn clear(&mut self);
    fn has_value(&self) -> bool;
}
#[derive(Default)]
pub struct DisabledCredentialStore(Option<String>);
impl CredentialStore for DisabledCredentialStore {
    fn put(&mut self, value: &str) {
        self.0 = Some(value.into())
    }
    fn clear(&mut self) {
        self.0 = None
    }
    fn has_value(&self) -> bool {
        self.0.is_some()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disconnect_clears_credentials() {
        let mut s = DisabledCredentialStore::default();
        s.put("secret");
        assert!(s.has_value());
        s.clear();
        assert!(!s.has_value());
    }
}
