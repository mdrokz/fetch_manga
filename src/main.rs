// #![feature(async_closure)]

use std::fs::OpenOptions;
use std::future::Future;
use std::io::Read;
use std::path::Path;
use std::{io::Write, time::Duration};

use async_once::AsyncOnce;
use fantoccini::{Client, ClientBuilder, Locator};
use std::fs::{create_dir_all, read_dir, File};

#[macro_use]
extern crate lazy_static;

const BASE_URL: &str = "https://mangasee123.com";

const CHECKPOINT_FILE: &str = "checkpoint.txt";

#[macro_use]
mod json;

use crate::json::Json;
use crate::json::Value;
use std::fmt::Display;

pub mod screenshot;

json! {
    Chapter,
    name => String,
    images => String
}

lazy_static! {
    static ref CLIENT: AsyncOnce<Client> = AsyncOnce::new(async {
        ClientBuilder::native()
            .connect("http://localhost:9515")
            .await
            .expect("failed to connect to WebDriver")
    });
}

#[derive(Debug, Default, Clone)]
enum MangaType {
    #[default]
    Manga,
    Manhwa,
    Manhua,
}

impl Into<MangaType> for String {
    fn into(self) -> MangaType {
        match self.as_str() {
            "manga" => MangaType::Manga,
            "manhwa" => MangaType::Manhwa,
            "manhua" => MangaType::Manhua,
            _ => MangaType::Manga,
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

async fn async_filter<'a: 'b, 'b, T: 'a + 'b + Clone, F: Future<Output = bool>>(
    a: &'a Vec<T>,
    p: impl Fn(&'b T) -> F,
) -> Vec<T>
// where
//     P: Fn(&T) -> Pin<Box<dyn Future<Output = bool>>>,
{
    let mut new = vec![];

    for x in a.iter() {
        if p(x).await {
            new.push(x.clone());
        }
    }
    return new;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();

    args.next();

    let manga_name = args
        .next()
        .map_or(Err("Argument manga name is missing"), |f| Ok(f))?;

    let manga_type = args
        .next()
        .map(|f| (Into::<MangaType>::into(f)))
        .map_or(Err("Argument manga type is missing"), |f| Ok(f))?;

    let manga_dir = format!(
        "./Dataset/{}/{}",
        <MangaType as Into<String>>::into(manga_type),
        manga_name
    );


    Ok(())
}
