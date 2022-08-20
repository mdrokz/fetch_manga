// #![feature(async_closure)]

use std::collections::VecDeque;
use std::future::Future;
use std::{io::Write, time::Duration};

use async_once::AsyncOnce;
use fantoccini::{Client, ClientBuilder, Locator};
use std::fs::{create_dir_all, read_dir};

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

    let mut chapters = vec![];

    chapters.push(Chapter {
        name: Value::String(format!("Chapter_{}", 1)),
        images: Value::Array(vec!["kek".into()]),
    });

    chapters.push(Chapter {
        name: Value::String(format!("Chapter_{}", 2)),
        images: Value::Array(vec!["kek".into()]),
    });

    // let c = Value::Array(chapters);

    // println!("{}",c);


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

    if let Err(_) = read_dir(&manga_dir) {
        create_dir_all(&manga_dir)?;
    }

    let c = CLIENT.get().await;

    let mut chapters = vec![];
    let mut index = 1;

    c.goto(&format!("{}/manga/{}", BASE_URL, manga_name))
        .await?;

    let show_all_chapters = c.find(Locator::Css(".ShowAllChapters")).await?;

    show_all_chapters.click().await?;

    let chapter_link_elements = c.find_all(Locator::Css(".top-10>a")).await?;

    let mut chapter_links = VecDeque::with_capacity(chapter_link_elements.len());

    for chapter_link in chapter_link_elements {
        chapter_links.push_front(tokio::spawn(async move {
            let link = chapter_link
                .attr("href")
                .await
                .map_or(String::new(), |f| f.expect("failed to extract href"));
            link
        }));
    }

    for chapter in chapter_links {
        let href = chapter.await?;

        if !href.contains("read-online") {
            continue;
        }

        tokio::spawn(async move {
            c.goto(&format!("{}{}", BASE_URL, href))
                .await
                .expect("failed to navigate")
        });

        tokio::time::sleep(Duration::from_millis(5000)).await;

        let button = &c.find_all(Locator::Css(".Column>.btn-sm")).await?[3];

        button.click().await?;

        let images = c.find_all(Locator::Css(".ng-scope>div>img")).await?;

        let mut image_links = vec![];

        let chapter_dir = format!("{}/Chapters/Chapter_{}", &manga_dir, index);

        if let Err(_) = read_dir(&chapter_dir) {
            create_dir_all(&chapter_dir)?;
        }

        for image in images {
            let src = image.attr("src").await?;

            if let Some(link) = src {
                // let name = link
                //     .split("/")
                //     .last()
                //     .ok_or("chapter name is missing from link")?;

                image_links.push(link);
            }
        }

        chapters.push(Chapter {
            name: Value::String(format!("Chapter_{}", index)),
            images: Value::Array(image_links),
        });

        index += 1;
    }

    // let chapters: Vec<String> = chapters.iter().map(|x| x.serialize()).collect();
    let chapters = Value::Array(chapters);

    let mut json_file = std::fs::File::create(format!("{}/chapters.json",manga_dir))?;

    json_file.write_all(format!("{}", chapters).replace("\\", "").as_bytes())?;

    Ok(())
}
