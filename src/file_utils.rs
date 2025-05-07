use std::fs::create_dir_all;
use glob::glob;
use std::{fs, path::Path, sync::mpsc::channel};
use notify::{RecursiveMode, Watcher};
use tracing::{info, warn};
use crate::config::Config;
use crate::html_generator;

pub fn create_files(config: &Config, md_files: Vec<String>) {
    clear_html_files(&config.webpage_dir);
    html_generator::create_blog_posts(config, md_files.clone());
    html_generator::create_index_page(config, md_files);
}

pub fn watch_for_changes(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let (_tx, rx) = channel::<String>();

    let config_clone = config.clone();
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            info!("{:?}", event);
            let md_files = read_markdown_files(&config_clone.data_dir);
            create_files(&config_clone, md_files);
        }
        Err(e) => warn!("watch error: {:?}", e),
    })?;

    watcher.watch(Path::new(&config.data_dir), RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(_) => {}
            Err(e) => warn!("watch receive error: {:?}", e),
        }
    }
}

// Read markdown files from a folder
pub fn read_markdown_files(folder: &str) -> Vec<String> {
    let pattern = format!("{}/*.md", folder);
    glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .map(|path| path.display().to_string())
        .collect()
}

// clear old HTML files
pub fn clear_html_files(webpage_dir: &str) {
    info!("Clearing old files");
    let pattern = format!("{}/*.html", webpage_dir);
    for entry in glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                fs::remove_file(path).expect("Error removing file");
            }
            Err(e) => warn!("{:?}", e),
        }
    }
}

// Helper function to move the image to the ./webpage/images/ directory
pub fn move_image_to_webpage(image_path: &str, images_dir_str: &str) {
    let images_dir = Path::new(images_dir_str);
    create_dir_all(&images_dir).expect("Failed to create target directory for images");
    
    let image_filename = Path::new(image_path).file_name().unwrap_or_else(|| 
        panic!("Failed to get filename from image_path: {}", image_path)
    );
    // Convert OsStr filename to str, panicking if not possible (should be rare for typical image names)
    let image_filename_str = image_filename.to_str().unwrap_or_else(|| 
        panic!("Failed to convert image filename to string for path: {:?}", image_filename)
    );

    // Correctly join the destination directory path with the image filename
    let target_path = images_dir.join(image_filename_str);
    let target_path_display = target_path.display().to_string(); // For logging

    if !target_path.exists() {
        match fs::copy(image_path, &target_path) {
            Ok(_) => info!("Successfully copied image from {} to {}", image_path, target_path_display),
            Err(e) => warn!("Failed to copy image from {} to {}: {}", image_path, target_path_display, e),
        }
    }
}

// Helper function to get the cover image HTML
pub fn get_cover_image_html(md_file: &str, config: &Config) -> String {
    let base_filename = Path::new(md_file).file_stem().unwrap_or_default().to_str().unwrap_or_default();
    if base_filename.is_empty() {
        warn!("Could not determine base filename from md_file: {}", md_file);
        return String::new();
    }

    let source_images_dir = Path::new(config.images_dir.trim_end_matches('/')); 
    let webpage_images_dest_dir_str = Path::new(config.webpage_dir.trim_end_matches('/')).join("images").to_str().unwrap_or_default().to_string();
    if webpage_images_dest_dir_str.is_empty() {
        warn!("Could not construct webpage_images_dest_dir_str");
        return String::new();
    }

    let extensions = ["jpg", "jpeg", "png", "gif"];
    for ext in &extensions {
        let image_filename_with_ext = format!("{}.{}", base_filename, ext);
        let source_image_full_path = source_images_dir.join(&image_filename_with_ext);

        if source_image_full_path.exists() {
            // Ensure webpage_images_dest_dir exists before calling move_image_to_webpage
            let dest_dir_path = Path::new(&webpage_images_dest_dir_str);
            if !dest_dir_path.exists() {
                create_dir_all(&dest_dir_path).expect("Failed to create webpage images destination directory");
            }
            move_image_to_webpage(source_image_full_path.to_str().unwrap_or_default(), &webpage_images_dest_dir_str);
            
            let html_image_src = Path::new("images").join(image_filename_with_ext);
            return format!(
                r#"<img class="cover-image" src="{}" alt="Cover Image">"#,
                html_image_src.to_str().unwrap_or_default()
            );
        }
    }
    String::new()
}

// Helper function to get the thumbnail meta tag and move the image
pub fn get_thumbnail_meta_tag(md_file: &str, config: &Config) -> String {
    let base_filename = Path::new(md_file).file_stem().unwrap_or_default().to_str().unwrap_or_default();
    if base_filename.is_empty() {
        warn!("Could not determine base filename for thumbnail from md_file: {}", md_file);
        return String::new();
    }

    let source_images_dir = Path::new(config.images_dir.trim_end_matches('/')); 
    let webpage_images_dest_dir_str = Path::new(config.webpage_dir.trim_end_matches('/')).join("images").to_str().unwrap_or_default().to_string();
    if webpage_images_dest_dir_str.is_empty() {
        warn!("Could not construct webpage_images_dest_dir_str for thumbnail");
        return String::new();
    }
    const WEBPAGE_IMAGES_SUBDIR_NAME: &str = "images";

    let extensions = ["jpg", "jpeg", "png", "gif"];
    for ext in &extensions {
        let image_filename_with_ext = format!("{}.{}", base_filename, ext);
        let source_image_full_path = source_images_dir.join(&image_filename_with_ext);

        if source_image_full_path.exists() {
            let dest_dir_path = Path::new(&webpage_images_dest_dir_str);
            if !dest_dir_path.exists() {
                create_dir_all(&dest_dir_path).expect("Failed to create webpage images destination directory");
            }
            move_image_to_webpage(source_image_full_path.to_str().unwrap_or_default(), &webpage_images_dest_dir_str);

            let thumbnail_url_path = Path::new(WEBPAGE_IMAGES_SUBDIR_NAME).join(image_filename_with_ext);
            let thumbnail_url = format!("{}/{}", config.base_url.trim_end_matches('/'), thumbnail_url_path.to_str().unwrap_or_default());
            return format!(r#"<meta property="og:image" content="{}"/>"#, thumbnail_url);
        }
    }
    String::new()
}
