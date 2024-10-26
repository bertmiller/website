use std::fs::create_dir_all;
use glob::glob;
use std::{fs, path::Path};

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
    println!("Clearing old files");
    let pattern = format!("{}/*.html", webpage_dir);
    for entry in glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                fs::remove_file(path).expect("Error removing file");
            }
            Err(e) => println!("{:?}", e),
        }
    }
}

// Helper function to move the image to the ./webpage/images/ directory
pub fn move_image_to_webpage(image_path: &str, images_dir: &str) {
    create_dir_all(images_dir).expect("Failed to create target directory");
    let image_filename = Path::new(image_path).file_name().unwrap().to_str().unwrap();
    let target_path = format!("{}{}", images_dir, image_filename);

    if !Path::new(&target_path).exists() {
        fs::copy(image_path, &target_path).expect("Failed to copy image file");
    }
}

// Helper function to get the cover image HTML
pub fn get_cover_image_html(md_file: &str, images_dir: &str) -> String {
    let base_filename = Path::new(md_file).file_stem().unwrap().to_str().unwrap();
    let extensions = ["jpg", "jpeg", "png", "gif"];
    for ext in &extensions {
        let image_path = format!("./images/{}.{}", base_filename, ext);
        if Path::new(&image_path).exists() {
            move_image_to_webpage(&image_path, &images_dir);
            return format!(
                r#"<img class="cover-image" src="{}" alt="Cover Image">"#,
                image_path
            );
        }
    }
    String::new()
}

// Helper function to get the thumbnail meta tag and move the image
pub fn get_thumbnail_meta_tag(md_file: &str, base_url: &String, images_dir: &str) -> String {
    let base_filename = Path::new(md_file).file_stem().unwrap().to_str().unwrap();
    let extensions = ["jpg", "jpeg", "png", "gif"];
    for ext in &extensions {
        let image_path = format!("./images/{}.{}", base_filename, ext);
        if Path::new(&image_path).exists() {
            move_image_to_webpage(&image_path, &images_dir);
            let thumbnail_url = format!("{}/{}", base_url, image_path.replace("./", ""));
            return format!(r#"<meta property="og:image" content="{}"/>"#, thumbnail_url);
        }
    }
    String::new()
}
