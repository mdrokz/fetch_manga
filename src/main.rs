// #![feature(async_closure)]

use async_std::prelude::StreamExt;
use chromiumoxide::{cdp::browser_protocol::target::CreateTargetParams, Browser, BrowserConfig};
// use scraper::{Html, Selector};

const BASE_URL: &str = "https://mangasee123.com";

#[derive(Debug, Clone)]
pub struct Chapter {
    name: String,
    images: Vec<String>,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();

    args.next();

    let manga_name = args
        .next()
        .map_or(Err("Argument manga name is missing"), |f| Ok(f))?;

    let (browser, mut handler) =
        Browser::launch(BrowserConfig::builder().with_head().build()?).await?;

    let handle = async_std::task::spawn(async move {
        loop {
            let _event = handler.next().await.unwrap();
        }
    });

    let page = &browser
    .new_page(format!(
        "{}/manga/{}",
        BASE_URL, manga_name
    ))
    .await?;

    page.evaluate("window.stop()").await?;

    let mut chapters = vec![];

    let show_all_chapters = page.find_element(".ShowAllChapters").await?;

    show_all_chapters.click().await?;

    let chapter_link_elements = page.find_elements(".top-10>a").await?;

    for (i, chapter) in chapter_link_elements.iter().enumerate() {
        let href = chapter.attribute("href").await?;

        if let Some(link) = href {
            let image_page = browser
                .new_page(CreateTargetParams::new(format!("{}{}", BASE_URL, link)))
                .await?;

            let button = &image_page.find_elements(".Column>.btn-sm").await?[3];

            button.click().await?;

            let images = image_page.find_elements(".ng-scope>div>img").await?;

            let mut image_links = vec![];

            for image in images {
                let src = image.attribute("src").await?;

                if let Some(link) = src {
                    image_links.push(link);
                }
            }

            chapters.push(Chapter {
                name: format!("Chapter {}", i),
                images: image_links,
            });

            println!("{:?}",chapters);

            image_page.close().await?;
        }
    }

    println!("{:?}", chapters);

    handle.await;

    Ok(())
}
