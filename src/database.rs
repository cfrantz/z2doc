use crate::models::{DisassemblyInfo, ThemeConfig};
use std::fs;
use std::path::Path;

pub fn load_db<P: AsRef<Path>>(path: P) -> Result<DisassemblyInfo, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let db: DisassemblyInfo = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(db)
}

pub fn save_db<P: AsRef<Path>>(path: P, db: &DisassemblyInfo) -> Result<(), String> {
    let content = serde_json::to_string_pretty(db).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_theme<P: AsRef<Path>>(path: P) -> Result<ThemeConfig, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let theme: ThemeConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(theme)
}

pub fn save_theme<P: AsRef<Path>>(path: P, theme: &ThemeConfig) -> Result<(), String> {
    let content = serde_json::to_string_pretty(theme).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}
