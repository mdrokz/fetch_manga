const BASE_URL: &str = "https://mangasee123.com/read-online";

#[tokio::main]
async fn main() -> Result<(), String> {
    let mut args = std::env::args();

    args.next();

    let manga_name = args
        .next()
        .map_or(Err("Argument manga name is missing"), |f| Ok(f))?;

    println!("{}",manga_name);

    Ok(())
}
