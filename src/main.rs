mod config;
mod file_utils;
mod html_generator;

use config::Config;

use notify::{RecursiveMode, Watcher};
use std::{path::Path, sync::mpsc::channel};

fn create_files(config: &Config, md_files: Vec<String>) {
    file_utils::clear_html_files(&config.webpage_dir);
    html_generator::create_blog_posts(config, md_files.clone());
    html_generator::create_index_page(config, md_files);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()?;
    let md_files = file_utils::read_markdown_files(&config.data_dir);
    create_files(&config, md_files.clone());
    watch_for_changes(&config)?;

    Ok(())
}

fn watch_for_changes(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let (_tx, rx) = channel::<String>();

    let config_clone = config.clone();
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            println!("{:?}", event);
            let md_files = file_utils::read_markdown_files(&config_clone.data_dir);
            create_files(&config_clone, md_files);
        }
        Err(e) => println!("watch error: {:?}", e),
    })?;

    watcher.watch(Path::new(&config.data_dir), RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(_) => {}
            Err(e) => println!("watch receive error: {:?}", e),
        }
    }
}