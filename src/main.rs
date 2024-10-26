mod config;
mod file_utils;
mod html_generator;

use config::Config;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()?;
    let md_files = file_utils::read_markdown_files(&config.data_dir);
    file_utils::create_files(&config, md_files.clone());
    file_utils::watch_for_changes(&config)?;

    Ok(())
}

