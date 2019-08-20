use std::io::{*, Stdout};
use std::env;
use nh_fetcher;
use log;
use simplelog::*;
use dialoguer;

fn main() {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info, 
        simplelog::Config::default(), 
        simplelog::TerminalMode::default()
    ).unwrap();

    let args: Vec<String> = env::args().collect();
    let mut fetch_url: String;
    let mut link_arg_present: bool;
    if args.len() < 2 {
        fetch_url= dialoguer::Input::new()
        .with_prompt("Enter album link").interact()
        .expect("Failed to read user input");
        link_arg_present = false;
    } else {
        fetch_url = args.last().unwrap().to_string();
        link_arg_present = true;
    }

    let (title, urls) = nh_fetcher::fetch_urls(&fetch_url).unwrap();
    log::info!("Fetching {}", title);
    let (success, total) = nh_fetcher::fetch_to_dir(urls, &title, true)
        .expect("Fetch failure");
        
    log::info!("Successfully downloaded {} out of {} images", success, total);
    
    if !link_arg_present {
        write!(stdout(), "Press any key to continue...").unwrap();
        stdout().flush().unwrap();
        stdin().read(&mut [0u8]).unwrap();
    }
}
