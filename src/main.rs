#![feature(async_closure)]

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

    let mut checkpoint_file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .create(true)
        .open(Path::new(&manga_dir).join(CHECKPOINT_FILE))?;

    let checkpoint_text = {
        let mut buf = String::new();
        let _ = checkpoint_file.read_to_string(&mut buf).map_or(0, |s| s);
        buf
    };

    let mut checkpoint_split = checkpoint_text.split("\n");

    let checkpoint_link = checkpoint_split.next().map_or("", |f| f);

    if let Err(_) = read_dir(&manga_dir) {
        create_dir_all(&manga_dir)?;
    }

    let c = CLIENT.get().await;

    let mut chapter_len = if checkpoint_text.is_empty() {
        (0, 0)
    } else {
        let mut checkpoint = checkpoint_split
            .next()
            .expect("failed to get next element for checkpoint")
            .split(",");

        (
            checkpoint.next().map_or(0, |f| {
                f.parse::<usize>().expect("failed to parse chapter count")
            }),
            checkpoint.next().map_or(0, |f| {
                f.parse::<usize>().expect("failed to parse chapter count")
            }),
        )
    };
    let mut chapters = vec![];
    let mut index = 1;

    if checkpoint_text.is_empty() {
        c.goto(&format!("{}/manga/{}", BASE_URL, manga_name))
            .await?;

        let show_all_chapters = c.find(Locator::Css(".ShowAllChapters")).await?;

        show_all_chapters.click().await?;

        let chapter_link_elements = c
            .find_all(Locator::Css(r#".top-10>a[class*="ChapterLink"]"#))
            .await?;

        chapter_len.1 = chapter_link_elements.len();

        c.goto(
            &chapter_link_elements
                .last()
                .expect("failed to get 1st chapter link")
                .attr("href")
                .await?
                .map_or(String::new(), |f| f),
        )
        .await?;
        // for chapter_link in chapter_link_elements {
        //     chapter_links.push_front(tokio::spawn(async move {
        //         let link = chapter_link
        //             .attr("href")
        //             .await
        //             .map_or(String::new(), |f| f.expect("failed to extract href"));
        //         link
        //     }));
        // }
    } else {
        c.goto(checkpoint_link).await?;
    }

    for chapter in chapter_len.0..chapter_len.1 {
        let url = c.current_url().await?;

        checkpoint_file
            .write_all(format!("{}\n{},{}\n", url, chapter, chapter_len.1).as_bytes())?;

        let btn_elements = &c.find_all(Locator::Css(".nav-link")).await?;

        // let filtered_btn = async_filter(&btn_elements.clone(), async move |e| {
        //     let t = e.attr("ng-click").await.map_or(String::new(), |f| if let Some(f) = f {
        //         f
        //     } else {
        //         String::new()
        //     });

        //     t.contains("vm.Next")
        // })
        // .await;

        let next_btn = btn_elements[4].clone();

        // let next_btn = filtered_btn.first().ok_or("failed to get first element")?;

        // let href = chapter.await?;

        // if !href.contains("read-online") {
        //     continue;
        // }
        // tokio::spawn(async move {
        //     c.goto(&format!("{}{}", BASE_URL, href))
        //         .await
        //         .expect("failed to navigate")
        // });
        let button = &c.find_all(Locator::Css(".Column>.btn-sm")).await?[3];

        let btn_text = button.text().await?;

        if btn_text.contains("Long Strip") {
            button.click().await?;
        }

        tokio::time::sleep(Duration::from_millis(5000)).await;

        let images = c.find_all(Locator::Css(".ng-scope>div>img")).await?;

        let mut image_links = vec![];

        let chapter_dir = format!("{}/Chapters/Chapter_{}", &manga_dir, index);

        if let Err(_) = read_dir(&chapter_dir) {
            create_dir_all(&chapter_dir)?;
        }

        for image in images {
            let src = image.attr("src").await?;

            if let Some(link) = src {
                let name = link
                    .split("/")
                    .last()
                    .ok_or("chapter name is missing from link")?;

                match image.screenshot().await {
                    Ok(bytes) => {
                        let mut chapter_file = File::create(format!(
                            "{}/Chapters/Chapter_{}/{}",
                            &manga_dir, index, name
                        ))?;
                        chapter_file.write_all(&bytes)?;

                        image_links.push(link);
                    }
                    Err(_) => {
                        println!("failed to screenshot image {}", link);
                    }
                };
            }
        }

        chapters.push(Chapter {
            name: Value::String(format!("Chapter_{}", index)),
            images: Value::Array(image_links),
        });

        index += 1;

        // c.execute(r#"[...document.getElementsByClassName("nav-link")].filter(x => x.innerText.includes("Chapter"))[1].click()"#, vec![]).await?;

        next_btn.click().await?;
    }

    let chapters: Vec<String> = chapters.iter().map(|x| x.serialize()).collect();

    let mut json_file = std::fs::File::create("./chapters.json")?;

    json_file.write_all(format!("{:?}", chapters).replace("\\", "").as_bytes())?;

    Ok(())
}
