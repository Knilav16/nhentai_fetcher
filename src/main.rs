use std::io::{stdout, stdin, Write, Read};
use std::env;
use nh_fetcher;
use log;

fn main() {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info, 
        simplelog::Config::default(), 
        simplelog::TerminalMode::default()
    ).unwrap();

    let args: Vec<String> = env::args().collect();
    let fetch_url: String;
    let path: String;
    let link_arg_present: bool;
    let save_path_present: bool;
    
    if args.len() < 2 {
        println!("Usage: nhentai_fetcher <URL> [path] (path defaults to ./<album title>)");
        std::process::exit(0);
    } else {
        fetch_url = args.get(2).unwrap().to_string();
        link_arg_present = true;
        if args.len() == 3 {
            path = args.last().unwrap().to_string();
            save_path_present = true;
        } else {
            path = "".to_string();
            save_path_present = false;
        }
    }

    let (title, urls) = nh_fetcher::fetch_urls(&fetch_url).unwrap();
    log::info!("Fetching {}", title);

    let save_path: String;
    if save_path_present {
        save_path = path;
    } else {
        save_path = title;
    }

    let (success, total) = nh_fetcher::fetch_to_dir(urls, &save_path, true)
        .expect("Fetch failure");
        
    log::info!("Successfully downloaded {} out of {} images", success, total);
    
    if !link_arg_present { // If not link is provided in args, we suppose the program was launched 
        write!(stdout(), "Press any key to continue...").unwrap();
        stdout().flush().unwrap();
        stdin().read(&mut [0u8]).unwrap();
    }
}
