use std::{
    fs::{create_dir_all, read_dir, OpenOptions, File},
    io::{Read, Write},
    path::Path, time::Duration,
};

use fantoccini::{Client, Locator};

use crate::{BASE_URL, CHECKPOINT_FILE, Chapter, json::Value};

pub async fn screenshot(
    manga_dir: &str,
    manga_name: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
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
        client.goto(&format!("{}/manga/{}", BASE_URL, manga_name))
            .await?;

        let show_all_chapters = client.find(Locator::Css(".ShowAllChapters")).await?;

        show_all_chapters.click().await?;

        let chapter_link_elements = client
            .find_all(Locator::Css(r#".top-10>a[class*="ChapterLink"]"#))
            .await?;

        chapter_len.1 = chapter_link_elements.len();

        client.goto(
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
        client.goto(checkpoint_link).await?;
    }

    for chapter in chapter_len.0..chapter_len.1 {
        let url = client.current_url().await?;

        checkpoint_file
            .write_all(format!("{}\n{},{}\n", url, chapter, chapter_len.1).as_bytes())?;

        let btn_elements = &client.find_all(Locator::Css(".nav-link")).await?;

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
        let button = &client.find_all(Locator::Css(".Column>.btn-sm")).await?[3];

        let btn_text = button.text().await?;

        if btn_text.contains("Long Strip") {
            button.click().await?;
        }

        tokio::time::sleep(Duration::from_millis(5000)).await;

        let images = client.find_all(Locator::Css(".ng-scope>div>img")).await?;

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
