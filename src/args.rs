use std::{
    borrow::Cow,
    fs::{read_to_string, remove_dir_all, DirEntry},
    io::Write,
    path::Path, str::FromStr,
};

use clap::{Args, Parser, Subcommand};

use std::fs;


// use crate::types::Manifest;

#[derive(Debug, Default, Clone)]
pub enum MangaType {
    #[default]
    Manga,
    Manhwa,
    Manhua,
}

impl FromStr for MangaType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "manga" => Ok(MangaType::Manga),
            "manhwa" => Ok(MangaType::Manhwa),
            "manhua" => Ok(MangaType::Manhua),
            _ => Ok(MangaType::Manga),
        }
    }
}

impl Into<String> for MangaType {
    fn into(self) -> String {
        match self {
            MangaType::Manga => "manga".into(),
            MangaType::Manhwa => "manhwa".into(),
            MangaType::Manhua => "manhua".into(),
        }
    }
}

#[derive(Debug, Parser)]
#[clap(author = "mdrokz", version = "1.0", about)]
pub struct FetchMangaArgs {
    #[clap(subcommand)]
    pub entity_type: EntityType,
}

#[derive(Debug, Subcommand)]
pub enum EntityType {
    Download(DownloadCommand),
    Scrape(ScrapeCommand)
}

#[derive(Debug, Args)]
pub struct ScrapeCommand {
    #[clap(short, long)]
    pub type_m: MangaType,
    #[clap(short, long)]
    pub manga_name: String,
}


#[derive(Debug, Args)]
pub struct DownloadCommand {
    #[clap(short, long)]
    pub type_m: MangaType,
    #[clap(short, long)]
    pub manga_name: String,
    #[clap(short,long)]
    pub count: Option<usize>
}
