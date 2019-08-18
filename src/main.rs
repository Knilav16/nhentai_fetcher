use nh_fetcher;
use log;
use simplelog::*;
use dialoguer;

fn main() {
    let fetch_url: String = dialoguer::Input::new()
        .with_prompt("Enter album link").interact()
        .expect("Failed to read user input");

    let (title, urls) = nh_fetcher::fetch_urls(&fetch_url).unwrap();
    log::info!("Fetching {}", title);
    let (success, total) = nh_fetcher::fetch_to_dir(urls, &title, true)
        .expect("Fetch failure");
        
    log::info!("Successfully downloaded {} out of {} images", success, total);
}
