use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::ExitCode,
};

use clap::Parser;
use indicatif::ProgressBar;
use md5::Digest;

use crate::{configuration::Configuration, models::DownloadLinks, utils::encode_hex};

mod configuration;
mod models;
mod utils;

fn main() -> ExitCode {
    dotenvy::dotenv().ok();
    const URL: &str =
        "https://remote-config.gog.com/components/webinstaller?component_version=2.0.0";

    const OS: &str = "windows";

    let config = Configuration::parse();

    println!("Configutration:");
    println!("{}", &config);

    let result = reqwest::blocking::get(URL);

    let response = match result {
        Ok(response) => response,
        Err(err) => {
            println!("Failed to get the download url from gog.");
            println!("Error: {}", err);
            return ExitCode::from(1);
        }
    };

    let download_links: DownloadLinks = match response.json() {
        Ok(json) => json,
        Err(err) => {
            println!("Failed to parse the download options.");
            println!("Error: {}", err);
            return ExitCode::from(2);
        }
    };

    let (download_url, download_size, download_hash) = match download_links.content.get(OS) {
        Some(download_option) => (
            &download_option.download_link,
            download_option.size,
            &download_option.installer_md5,
        ),
        None => {
            println!("Windows download option not found.");
            return ExitCode::from(3);
        }
    };

    let file_name = match download_url.split('/').last() {
        Some(value) => value,
        None => {
            println!("Malformed Downlaod URL. - {}", download_url);
            return ExitCode::from(4);
        }
    };

    let download_path = Path::new(&config.download_path).join(file_name);

    if !config.r#override && download_path.exists() {
        println!("File already exists.");
        return ExitCode::from(0);
    }

    if config.r#override && download_path.exists() {
        println!("File will be overriden.");
    }

    let mut download_response = match reqwest::blocking::get(download_url) {
        Ok(json) => json,
        Err(err) => {
            println!("Failed to download the gog client from {}.", download_url);
            println!("Error: {}", err);
            return ExitCode::from(5);
        }
    };

    if let Err(err) = fs::create_dir_all(config.download_path) {
        println!("Failed to create the download dir.");
        println!("Error: {}", err);
        return ExitCode::from(6);
    }

    let mut download_file = match File::create(&download_path) {
        Ok(file) => file,
        Err(err) => {
            println!("Failed to create file at download path.");
            println!("Error: {}", err);
            return ExitCode::from(7);
        }
    };

    println!("Starting download!");
    let mut buffer: [u8; 4096] = [0u8; 4096];

    let mut bar = ProgressBar::new(download_size);

    loop {
        let read_result = download_response.read(&mut buffer);

        let read = match read_result {
            Ok(read) => {
                if let Err(write_err) = download_file.write(&buffer[0..read]) {
                    println!("Failed to write data to file.");
                    println!("Error: {}", write_err);
                    return ExitCode::from(9);
                }
                bar.inc(read as u64);
                read
            }
            Err(err) => {
                println!("Failed to download file");
                println!("Error: {}", err);
                return ExitCode::from(10);
            }
        };

        if read == 0 {
            bar.finish();
            break;
        }
    }

    if let Err(err) = download_file.sync_all() {
        println!("Failed to sync file with filesystem.");
        println!("Error: {}", err);
        return ExitCode::from(11);
    }

    println!("Download successfully completed!");

    println!("Verifing Download.");


    let mut hash_file = match File::open(download_path) {
        Ok(file) => file,
        Err(err) => {
            println!("Failed to open file for verification!");
            println!("Error: {}", err);
            return ExitCode::from(13);
        }
    };

    let mut hasher = md5::Md5::new();
    bar = ProgressBar::new(download_size);
    loop {
        let read = match hash_file.read(&mut buffer) {
            Ok(read) => {
                hasher.update(&buffer[0..read]);
                bar.inc(read as u64);
                read
            },
            Err(err) => {
                println!("Failed to read downloaded file!");
                println!("Error: {}", err);
                return ExitCode::from(14);
            }
        };

        if read == 0 {
            bar.finish();
            break;
        }
    }

    let hash = hasher.finalize();
    let hash_hex = encode_hex(&hash);
    
    if &hash_hex != download_hash {
        println!("MD5-Hash differs between presumed({download_hash}) and downloaded({hash_hex}) file!");
        return ExitCode::from(15);
    }

    println!("Verification successful!");
    ExitCode::from(0)
}
