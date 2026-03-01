#[macro_use] extern crate rocket;

mod models;
mod disasm;
mod database;

use rocket::fs::NamedFile;
use std::path::{Path, PathBuf};

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/index.html")).await.ok()
}

#[get("/static/<file..>")]
async fn static_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).await.ok()
}

// TODO: API Endpoints
// - GET /api/disassembly/<bank_id>
// - POST /api/annotation
// - GET /api/theme.css
// - GET /api/metadata

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, static_files])
}
