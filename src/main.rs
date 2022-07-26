// #![feature(async_closure)]

use std::{collections::VecDeque, io::Write, time::Duration};

use async_once::AsyncOnce;
use fantoccini::{Client, ClientBuilder, Locator};
use std::fs::{create_dir, File, read_dir};

// use scraper::{Html, Selector};

#[macro_use]
extern crate lazy_static;

const BASE_URL: &str = "https://mangasee123.com";

#[macro_use]
mod json;

use crate::json::Json;
use crate::json::Value;
use std::fmt::Display;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();

    args.next();

    let manga_name = args
        .next()
        .map_or(Err("Argument manga name is missing"), |f| Ok(f))?;

    let c = CLIENT.get().await;

    c.goto(&format!("{}/manga/{}", BASE_URL, manga_name))
        .await?;

    let mut chapters = vec![];
    let mut index = 1;

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

        let chapter_dir = format!("Chapters/Chapter_{}", index);

        if let Err(_) = read_dir(&chapter_dir) {
            create_dir(&chapter_dir)?;
        }


        for image in images {
            let src = image.attr("src").await?;

            if let Some(link) = src {
                let name = link
                    .split("/")
                    .last()
                    .ok_or("chapter name is missing from link")?;

                let bytes = image.screenshot().await?;

                let mut chapter_file =
                    File::create(format!("Chapters/Chapter_{}/{}", index, name))?;

                chapter_file.write_all(&bytes)?;

                image_links.push(link);
            }
        }

        chapters.push(Chapter {
            name: Value::String(format!("Chapter_{}", index)),
            images: Value::Array(image_links),
        });

        index += 1;
    }

    let chapters: Vec<String> = chapters.iter().map(|x| x.serialize()).collect();

    let mut json_file = std::fs::File::create("./chapters.json")?;

    json_file.write_all(format!("{:?}", chapters).replace("\\", "").as_bytes())?;

    Ok(())
}
