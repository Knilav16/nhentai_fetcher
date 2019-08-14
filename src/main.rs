use nh_fetcher;
use log;
use dialoguer;
use simplelog::*;

fn main() {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Stdout)
        .expect("Failed to init the logger");

    
    let fetch_url = dialoguer::Input::<String>::new()
        .with_prompt("Enter album link")
        .interact()
        .unwrap();

    log::info!("Got link: {}", &fetch_url);

    let (title, urls) = nh_fetcher::fetch_urls(&fetch_url).unwrap();
    let (success, total) = nh_fetcher::fetch_to_dir(urls, &title, true)
        .unwrap();
        
    log::info!("Successfully downloaded {} out of {} images", success, total);
}
