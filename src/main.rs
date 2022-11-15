// #![feature(async_closure)]

use std::collections::{VecDeque};
use std::future::Future;
use std::{io::Write, time::Duration};

use args::{DownloadCommand, EntityType, FetchMangaArgs, MangaType, ScrapeCommand};
use async_once::AsyncOnce;
use clap::Parser;
use fantoccini::{Client, ClientBuilder, Locator};
use rusoto_core::{ByteStream, Region};
use rusoto_s3::{GetObjectRequest, ListObjectsV2Request, PutObjectRequest, S3Client, S3};
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

pub mod args;
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
    dotenv::dotenv()?;

    let bucket = std::env::var("S3_BUCKET")?;
    let region = std::env::var("S3_REGION")?;

    println!("AWS: {}", bucket);
    println!("AWS: {}", region);

    let s3_client = S3Client::new(Region::ApSouth1);

    let args = FetchMangaArgs::parse();

    match args.entity_type {
        EntityType::Download(DownloadCommand {
            manga_name,
            type_m,
            count,
        }) => {
            if let Some(count) = count {
            } else {
                let manga_dir = format!(
                    "Dataset/{}/{}/Chapters/",
                    <MangaType as Into<String>>::into(type_m),
                    manga_name
                );

                if let Err(_) = read_dir(&manga_dir) {
                    create_dir_all(&manga_dir)?;
                }

                let list_obj_req = ListObjectsV2Request {
                    bucket: bucket.clone(),
                    prefix: Some(manga_dir.clone()),
                    ..Default::default()
                };

                let o = s3_client.list_objects_v2(list_obj_req).await?;

                println!("{:?}", o);

                if let Some(contents) = o.contents {
                    for content in &contents {
                        let key = content.key.as_ref().map_or(String::new(), |f| f.clone());
                        let chapter = s3_client
                            .get_object(GetObjectRequest {
                                bucket: bucket.clone(),
                                key: key.clone(),
                                ..Default::default()
                            })
                            .await?;

                        let split = key.split("/");

                        let count = split.clone().count() - 2;

                        let vec = split.clone().collect::<Vec<&str>>();

                        let dir = vec.get(count).unwrap();

                        let name = split.last().expect("failed to get file name");

                        let stream = chapter.body.expect("failed to get stream");

                        let mut bytes = vec![];

                        let chapter_dir = format!("{}/{}", manga_dir, dir,);

                        if let Err(_) = read_dir(&chapter_dir) {
                            create_dir_all(&chapter_dir)?;
                        }

                        println!("Downloading {}", format!("{}/{}", chapter_dir, name));

                        let read =
                            tokio::io::copy(&mut stream.into_async_read(), &mut bytes).await?;

                        if read > 0 {
                            std::fs::write(format!("{}/{}", chapter_dir, name), bytes)?;
                        }
                    }
                }
            }
        }
        EntityType::Scrape(ScrapeCommand { manga_name, type_m }) => {
            let manga_dir = format!(
                "./Dataset/{}/{}",
                <MangaType as Into<String>>::into(type_m),
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

            let show_all_chapters = c.find(Locator::Css(".ShowAllChapters")).await;

            if let Ok(e) = show_all_chapters {
                e.click().await?;
            }

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

            let mut json_file = std::fs::File::create(format!("{}/chapters.json", manga_dir))?;

            let json = format!("{}", chapters).replace("\\", "");

            let res = s3_client
                .put_object(PutObjectRequest {
                    bucket: "machine-learning-objects".to_string(),
                    body: Some(ByteStream::from(json.as_bytes().to_vec())),
                    key: format!("{}/chapters.json", manga_dir).replace("./", ""),
                    ..Default::default()
                })
                .await?;

            println!("{:?}", res);

            json_file.write_all(json.as_bytes())?;
        }
    }

    Ok(())
}
