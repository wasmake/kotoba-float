use std::{fs, path::PathBuf};
pub fn load(path: &PathBuf) -> Option<serde_json::Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}
pub fn save(path: &PathBuf, value: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    let safe = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, safe).map_err(|e| e.to_string())
}
