#[macro_use] extern crate rocket;

mod models;
mod disasm;
mod database;

use crate::models::{DisassemblyInfo, DisassemblyLine, ThemeConfig};
use rocket::http::ContentType;
use clap::Parser;
use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;
use std::fs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the NES ROM file
    rom_path: PathBuf,

    /// Path to the database file (defaults to <rom_path>.json)
    #[arg(short, long)]
    db_path: Option<PathBuf>,

    /// Path to the theme configuration file (defaults to theme.json)
    #[arg(short, long)]
    theme_path: Option<PathBuf>,
}

pub struct AppState {
    pub db: RwLock<DisassemblyInfo>,
    pub db_path: PathBuf,
    pub themes: BTreeMap<String, ThemeConfig>,
    pub active_theme: RwLock<String>,
    pub theme_path: PathBuf,
    pub rom_data: Vec<u8>,
    pub rom_path: PathBuf,
}

#[derive(Serialize)]
pub struct Metadata {
    pub name: String,
    pub title: String,
    pub rom_file: String,
    pub banks: BTreeMap<u8, String>,
    pub mapper_window_size: u8,
}

#[derive(serde::Deserialize)]
pub struct AnnotationRequest {
    pub bank_id: Option<u8>,
    pub address: u16,
    pub symbol: Option<String>,
    pub comment: Option<String>,
    pub block_comment: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct ThemeRequest {
    pub name: String,
}

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/index.html")).await.ok()
}

#[get("/static/<file..>")]
async fn static_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).await.ok()
}

#[get("/api/metadata")]
async fn get_metadata(state: &State<Arc<AppState>>) -> Json<Metadata> {
    let db = state.db.read().await;
    let rom_name = state.rom_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let prg_banks_16k = state.rom_data[4];
    let num_banks = match db.mapper_window_size {
        8 => prg_banks_16k * 2,
        16 => prg_banks_16k,
        _ => prg_banks_16k,
    };

    let mut banks = BTreeMap::new();
    for i in 0..num_banks {
        let title = db.bank.get(&i)
            .and_then(|b| b.title.clone())
            .unwrap_or_else(|| String::new());
        banks.insert(i, title);
    }
    banks.insert(255, "Global Symbols".to_string());

    Json(Metadata {
        name: db.name.clone(),
        title: db.title.clone(),
        rom_file: rom_name,
        banks,
        mapper_window_size: db.mapper_window_size,
    })
}

#[get("/api/disassembly/<bank_id>")]
async fn get_disassembly(bank_id: u8, state: &State<Arc<AppState>>) -> Json<Vec<DisassemblyLine>> {
    let db = state.db.read().await;
    
    if bank_id == 255 {
        let mut lines = Vec::new();
        for (addr, anno) in &db.global {
            lines.push(DisassemblyLine {
                address: *addr,
                address_label: format!("${:04X}", addr),
                bank: -1,
                bytes: String::new(),
                opcode: String::new(),
                operand_prefix: String::new(),
                operand_main: String::new(),
                operand_suffix: String::new(),
                operand_is_symbol: false,
                symbol: anno.symbol.clone(),
                comment: anno.comment.clone(),
                block_comment: anno.block_comment.clone(),
                target_bank: None,
                target_address: None,
            });
        }
        return Json(lines);
    }

    let mapper_size = db.mapper_window_size as usize * 1024;
    let rom_offset = 16 + (bank_id as usize * mapper_size);
    let rom_end = (rom_offset + mapper_size).min(state.rom_data.len());
    
    let bank_data = if rom_offset < state.rom_data.len() {
        &state.rom_data[rom_offset..rom_end]
    } else {
        &[]
    };

    let lines = disasm::disassemble_bank(&db, bank_id, bank_data);
    Json(lines)
}

#[get("/api/themes")]
async fn get_themes(state: &State<Arc<AppState>>) -> Json<Vec<String>> {
    Json(state.themes.keys().cloned().collect())
}

#[post("/api/themes/active", data = "<req>")]
async fn set_active_theme(req: Json<ThemeRequest>, state: &State<Arc<AppState>>) -> Result<(), String> {
    if !state.themes.contains_key(&req.name) {
        return Err("Theme not found".to_string());
    }
    let mut active = state.active_theme.write().await;
    *active = req.name.clone();
    Ok(())
}

#[get("/api/theme.css")]
async fn get_theme_css(state: &State<Arc<AppState>>) -> (ContentType, String) {
    let active_name = state.active_theme.read().await;
    let theme = state.themes.get(&*active_name).unwrap();
    (ContentType::CSS, format!(
        "body {{ background-color: {}; color: {}; }}\n\
         .address {{ color: {}; }}\n\
         .hex {{ color: {}; }}\n\
         .instruction {{ color: {}; }}\n\
         .opcode {{ color: {}; }}\n\
         .operand {{ color: {}; }}\n\
         .comment {{ color: {}; }}\n\
         .symbol {{ color: {}; }}\n\
         a {{ color: inherit; text-decoration: none; }}\n\
         a.symbol {{ color: {}; }}\n\
         a:hover {{ text-decoration: underline; }}\n",
        theme.background, theme.instruction,
        theme.address, theme.hex, theme.instruction, theme.opcode,
        theme.instruction, theme.comment, theme.symbol, theme.symbol
    ))
}

#[post("/api/annotation", data = "<req>")]
async fn update_annotation(req: Json<AnnotationRequest>, state: &State<Arc<AppState>>) -> Result<(), String> {
    let mut db = state.db.write().await;
    
    let info = crate::models::AnnotationInfo {
        symbol: req.symbol.clone(),
        comment: req.comment.clone(),
        block_comment: req.block_comment.clone(),
    };

    if let Some(bank_id) = req.bank_id {
        let bank = db.bank.entry(bank_id).or_insert_with(|| crate::models::BankInfo {
            title: None,
            mapped_at: Some(0x8000),
            region: Vec::new(),
            address: std::collections::BTreeMap::new(),
        });
        bank.address.insert(req.address, info);
    } else {
        db.global.insert(req.address, info);
    }

    database::save_db(&state.db_path, &db).map_err(|e| e.to_string())?;
    Ok(())
}

#[launch]
async fn rocket() -> _ {
    let cli = Cli::parse();

    let rom_path = cli.rom_path;
    let db_path = cli.db_path.unwrap_or_else(|| {
        let mut p = rom_path.clone();
        p.set_extension("json");
        p
    });
    let theme_path = cli.theme_path.unwrap_or_else(|| PathBuf::from("theme.json"));

    let rom_data = fs::read(&rom_path).expect("Failed to read ROM file");
    
    let mut db = if db_path.exists() {
        database::load_db(&db_path).expect("Failed to load database")
    } else {
        let template_path = Path::new("templates/default_db.json");
        let mut default_db = database::load_db(template_path).expect("Failed to load default template");
        
        if !default_db.bank.contains_key(&0) {
            default_db.bank.insert(0, crate::models::BankInfo {
                title: Some("HUD & Overworld".to_string()),
                region: vec![crate::models::RegionInfo::Code(0x8000..=0xBFFF)],
                address: std::collections::BTreeMap::new(),
                mapped_at: Some(0x8000),
            });
        }
        
        default_db
    };

    if db.name.is_empty() {
        db.name = rom_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("project")
            .to_string();
    }
    if db.title.is_empty() {
        db.title = rom_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("NES Disassembly")
            .to_string();
    }

    database::save_db(&db_path, &db).expect("Failed to save database");

    let mut themes = BTreeMap::new();
    themes.insert("Light".to_string(), ThemeConfig {
        name: "Light".to_string(),
        background: "#FFFFFF".to_string(),
        address: "#808080".to_string(),
        hex: "#808080".to_string(),
        instruction: "#000000".to_string(),
        opcode: "#000000".to_string(),
        comment: "#808080".to_string(),
        symbol: "#0000FF".to_string(),
    });
    themes.insert("Dark".to_string(), ThemeConfig {
        name: "Dark".to_string(),
        background: "#202020".to_string(),
        address: "#909090".to_string(),
        hex: "#909090".to_string(),
        instruction: "#FFFFFF".to_string(),
        opcode: "#FFFFFF".to_string(),
        comment: "#909090".to_string(),
        symbol: "#0000FF".to_string(),
    });

    let mut active_theme = "Dark".to_string();

    if theme_path.exists() {
        let user_theme = database::load_theme(&theme_path).expect("Failed to load user theme");
        themes.insert("User".to_string(), user_theme);
        active_theme = "User".to_string();
    }

    let state = Arc::new(AppState {
        db: RwLock::new(db),
        db_path,
        themes,
        active_theme: RwLock::new(active_theme),
        theme_path,
        rom_data,
        rom_path,
    });

    rocket::build()
        .manage(state)
        .mount("/", routes![index, static_files, get_metadata, get_disassembly, update_annotation, get_theme_css, get_themes, set_active_theme])
}
