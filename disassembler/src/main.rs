use anyhow::{anyhow, Result};
use log::LevelFilter;
use std::path::PathBuf;
use structopt::StructOpt;

mod description;
mod dis;

use description::nesfile::NesFile;
use dis::rom::Rom;

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(short, long, help = "Description file")]
    description: PathBuf,

    #[structopt(short, long, help = "Parse descriprtion file only")]
    parse: bool,

    #[structopt(long, default_value = "off")]
    logging: LevelFilter,

    #[structopt(name = "NESFILE")]
    nesfile: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opts = Opts::from_args();
    env_logger::Builder::from_default_env()
        .filter(None, opts.logging)
        .init();

    let desc = std::fs::read_to_string(opts.description)?;
    let desc: NesFile = serde_yaml::from_str(&desc)?;
    if opts.parse {
        println!("{}", serde_yaml::to_string(&desc)?);
        return Ok(());
    }

    let nesfile = opts.nesfile.as_ref().or(desc.nesfile.as_ref()).ok_or(
        anyhow!("No nesfile specified.  Please specify a nesfile as a commandline argument or in the description file."))?.clone();

    let mut rom = Rom::new(&nesfile)?;
    rom.process(&desc)?;

    rom.to_text(&desc);
    Ok(())
}
