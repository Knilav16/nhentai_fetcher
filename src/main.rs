use nh_fetcher;
use log;
use simplelog::*;
use std::env;

fn main() {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Stdout)
        .expect("Failed to init the logger");

    let args: Vec<String> = env::args().collect();
    let fetch_url = &args[1];

    let (title, urls) = nh_fetcher::fetch_urls(&fetch_url).unwrap();
    log::info!("Fetching {}", title);
    let (success, total) = nh_fetcher::fetch_to_dir(urls, &title, true)
        .expect("Fetch failure");
        
    log::info!("Successfully downloaded {} out of {} images", success, total);
}
