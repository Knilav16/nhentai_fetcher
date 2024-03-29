use std::fs::{File, create_dir, read_dir, remove_dir_all};
use std::ffi::{OsString, OsStr};
use std::io::BufWriter;
use std::path::Path;
use printpdf::types::plugins::graphics::two_dimensional::image::Image;
use printpdf::{scale::Mm};
use printpdf;
use log;
use indicatif;
use dialoguer::Input;
use reqwest::{Client, StatusCode, header::{USER_AGENT, CONTENT_TYPE}};
use select::document::Document;
use select::predicate::{Predicate, Class, Name, Attr};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

/// Gets all the image links from the main page of a nhentai album
/// Returns the album title and all the links as a `Vector` of `String`s
/// # Arguments
/// * `fetch_url` - The URL of a nhentai album (i.e: https://nhentai.net/g/183861/)
pub fn fetch_urls(fetch_url: &str) -> Result<(String, Vec<String>), String> {
    let client = Client::new();

    let mut res = match client.get(fetch_url)
        .header(USER_AGENT, "nhentai Fetcher")
        .send() {
            Ok(res) => res,
            Err(e) => {
                log::error!("\"{}\" occured while fetching {}", e, fetch_url);
                return Result::Err(e.to_string());
            }
        };

    let raw_data = match res.status() {
        StatusCode::OK => {
            match res.text() {
                Ok(data) => data,
                Err(e) => {
                    log::error!("\"{}\" occured while trying to get raw text data from {}", e, fetch_url);
                    return Result::Err(e.to_string());
                }
            }          
        },

        _ => {
            log::error!("GET to {} did not return code 200", fetch_url);
            return Err(format!("Unvalid URL, status code: {}", res.status()));
        }
    };

    let main_page = Document::from(raw_data.as_str());

    // Finding album title
    let title = main_page.find(Attr("id", "info").descendant(Name("h1"))).next().unwrap().text();
    log::trace!("Found title: {}", title);

    let mut to_fetch: Vec<String> = Vec::new();

    // Finding all the links
    for node in main_page.find(Class("gallerythumb").descendant(Name("img"))) { // Searching for all thumb links on the main page and getting the image links
        // thumb_url example : https://t.nhentai.net/galleries/<number>/<number>t.jpg
        let thumb_url = node.attr("data-src").unwrap();
        
        to_fetch.push(
            // Replace the first 't' by 'i', we use '//t' to not match with the 't' in 'http'
            thumb_url.replacen("//t", "i", 1)
            // Replace the second 't.'(which is actually the first 't.' by now) by '.'
            .replacen("t.", ".", 1)
        );
    }

    Ok((title, to_fetch))
}

/// Downloads and saves the images from `urls` in `directory`
/// Returns how many images were saved out of how many images(as a pair of `usize`)
/// # Arguments
/// * `urls` - A string vector containing the urls of a nhentai album images(got using fetch_urls)
/// * `directory` - The name of the directory where the album will be saved
/// * `progress` - Tells if the function should show progression using `indicatif`
pub fn fetch_to_dir(urls: Vec<String>, directory: &str, progress: bool) -> Result<(usize, usize), String> {
    // Creating the album directory
    let mut final_directory = String::from(directory);
    if let Err(e) = create_dir(&final_directory) {
        log::warn!("Failed to create {}: \"{}\", removing invalid characters", &final_directory, e);
        let forbidden_chars = "/\\:*?\"<>|";
        
        final_directory.retain(|c| {
            !forbidden_chars.contains(c)
        });

        log::warn!("Attempting to create {}", &final_directory);

        if let Err(_) = create_dir(&final_directory) {
            log::warn!("Failed to correct directory name: {}", &final_directory);
            final_directory = Input::new()
                .with_prompt(&format!("Enter a valid name for {}", &final_directory))
                .interact()
                .expect("Failed to read user input");
            if let Err(e) = create_dir(&final_directory) {
                return Err(format!("Failed to create user inputted directory: {}", e));
            }
        }
    }
        
    let client = reqwest::Client::new();

    let mut success = 0; // Counting successful downloads
    // Fetching all the images and saving them on the disk

    let progress_bar = match progress {
        true => {
            let tmp = indicatif::ProgressBar::new(urls.len() as u64);
            tmp.set_style(
                indicatif::ProgressStyle::default_bar()
                .template("{elapsed_precise} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            );
            tmp
        },
        false => indicatif::ProgressBar::hidden()
    };

    for (i, l) in urls.iter().enumerate() {
        log::trace!("Fetching {}", l);
        let mut response = match client.get(l)
        .header(USER_AGENT, "nhentai Fetcher")
        .send() {
            Ok(res) => res,
            Err(e) => {
                log::error!("{} occured while fetching {}", e, l);
                continue;
            }
        };
        
        match response.status() {
            StatusCode::OK => {
                match response.headers().get(CONTENT_TYPE) {
                    Some(content_type) => {
                        let file_name = match content_type.to_str().unwrap() {
                            "image/png" => format!("{}/{}.png", &final_directory, i + 1),
                            "image/jpeg" => format!("{}/{}.jpg", &final_directory, i + 1),
                            _ => {
                                log::error!("GET to {} did not return a jpg or a png", l);
                                continue;
                            }
                        };

                        let mut image = File::create(&file_name)
                            .expect(&format!("Failed to create {}", file_name));
                        if let Ok(written) = response.copy_to(&mut image) {
                            log::trace!("Written {} bytes", written);
                            success += 1;
                        }
                    },
                    None => continue
                }
            },

            _ => {
                log::error!("GET to {} did not return code 200", l);
                continue;
            }
        }
        progress_bar.inc(1);
    }
    progress_bar.finish_with_message(&format!("Successfully downloaded {} images", urls.len()));

    Ok((success, urls.len()))
}
