use crate::models::{DisassemblyInfo, ThemeConfig};

// In WASM, we don't use std::fs. 
// These functions are kept for structural compatibility if needed, 
// but they now take/return strings instead of paths.

pub fn parse_db(content: &str) -> Result<DisassemblyInfo, String> {
    let db: DisassemblyInfo = serde_json::from_str(content).map_err(|e| e.to_string())?;
    Ok(db)
}

pub fn serialize_db(db: &DisassemblyInfo) -> Result<String, String> {
    serde_json::to_string_pretty(db).map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub fn parse_theme(content: &str) -> Result<ThemeConfig, String> {
    let theme: ThemeConfig = serde_json::from_str(content).map_err(|e| e.to_string())?;
    Ok(theme)
}
