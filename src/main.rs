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
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the NES ROM file
    rom_path: PathBuf,

    /// Path to the database file (defaults to <rom_path>.json5)
    #[arg(short, long)]
    db_path: Option<PathBuf>,

    /// Path to the theme configuration file (defaults to theme.json5)
    #[arg(short, long)]
    theme_path: Option<PathBuf>,
}

pub struct AppState {
    pub db: RwLock<DisassemblyInfo>,
    pub db_path: PathBuf,
    pub theme: RwLock<ThemeConfig>,
    pub theme_path: PathBuf,
    pub rom_data: Vec<u8>,
    pub rom_path: PathBuf,
}

#[derive(Serialize)]
pub struct Metadata {
    pub rom_file: String,
    pub total_banks: u8,
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
    let total_banks = match db.mapper_window_size {
        8 => prg_banks_16k * 2,
        16 => prg_banks_16k,
        _ => prg_banks_16k,
    };

    Json(Metadata {
        rom_file: rom_name,
        total_banks,
        mapper_window_size: db.mapper_window_size,
    })
}

#[get("/api/disassembly/<bank_id>")]
async fn get_disassembly(bank_id: u8, state: &State<Arc<AppState>>) -> Json<Vec<DisassemblyLine>> {
    let db = state.db.read().await;
    
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

#[get("/api/theme.css")]
async fn get_theme_css(state: &State<Arc<AppState>>) -> (ContentType, String) {
    let theme = state.theme.read().await;
    (ContentType::CSS, format!(
        "body {{ background-color: {}; color: {}; }}\n\
         .address {{ color: {}; }}\n\
         .hex {{ color: {}; }}\n\
         .instruction {{ color: {}; }}\n\
         .opcode {{ color: {}; }}\n\
         .operand {{ color: {}; }}\n\
         .comment {{ color: {}; }}\n\
         .symbol {{ color: {}; }}\n\
         a {{ color: {}; text-decoration: none; }}\n\
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
    
    let db = if db_path.exists() {
        database::load_db(&db_path).expect("Failed to load database")
    } else {
        let template_path = Path::new("templates/default_db.json");
        let mut default_db = database::load_db(template_path).expect("Failed to load default template");
        
        if !default_db.bank.contains_key(&0) {
            default_db.bank.insert(0, crate::models::BankInfo {
                region: vec![crate::models::RegionInfo::Code(0x8000..=0xBFFF)],
                address: std::collections::BTreeMap::new(),
                mapped_at: Some(0x8000),
            });
        }
        
        database::save_db(&db_path, &default_db).expect("Failed to save initial database");
        default_db
    };

    let theme = if theme_path.exists() {
        database::load_theme(&theme_path).expect("Failed to load theme")
    } else {
        let default_theme = ThemeConfig {
            name: "Default Dark".to_string(),
            background: "#202020".to_string(),
            address: "#909090".to_string(),
            hex: "#909090".to_string(),
            instruction: "#FFFFFF".to_string(),
            opcode: "#FFFFFF".to_string(),
            comment: "#909090".to_string(),
            symbol: "#0000FF".to_string(),
        };
        database::save_theme(&theme_path, &default_theme).expect("Failed to save initial theme");
        default_theme
    };

    let state = Arc::new(AppState {
        db: RwLock::new(db),
        db_path,
        theme: RwLock::new(theme),
        theme_path,
        rom_data,
        rom_path,
    });

    rocket::build()
        .manage(state)
        .mount("/", routes![index, static_files, get_metadata, get_disassembly, update_annotation, get_theme_css])
}
