mod config;

use chrono::NaiveDate;
use config::Config;

use glob::glob;
use notify::{RecursiveMode, Watcher};
use pulldown_cmark::{html::push_html, Options, Parser};
use std::fs::create_dir_all;
use std::{fs, path::Path, sync::mpsc::channel};

fn create_files(config: &Config, md_files: Vec<String>) {
    clear_html_files(&config.webpage_dir);
    create_html_files(config, md_files.clone());
    create_index_page(config, md_files);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()?;

    println!("Using base_url: {}", config.base_url);
    println!("Using title: {}", config.title);

    let md_files = read_markdown_files(&config.data_dir);
    create_files(&config, md_files.clone());

    let (_tx, rx) = channel::<String>();

    let config_clone = config.clone();
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            println!("{:?}", event);
            let md_files = read_markdown_files(&config_clone.data_dir);
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

// Read markdown files from a folder
fn read_markdown_files(folder: &str) -> Vec<String> {
    let pattern = format!("{}/*.md", folder);
    glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .map(|path| path.display().to_string())
        .collect()
}

// Convert markdown to HTML
fn markdown_to_html(markdown: &str, md_file: &str, config: &Config) -> String {
    // Extract the title (first h2) and date (second line)
    let mut lines = markdown.lines();
    let title = lines
        .next()
        .unwrap_or("")
        .trim_start_matches("## ")
        .to_string();
    let date = lines.next().unwrap_or("").to_string();

    // Skip any empty lines after the date
    let content = lines
        .skip_while(|line| line.trim().is_empty())
        .collect::<Vec<&str>>()
        .join("\n");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&content, options);
    let mut html_output = String::new();
    push_html(&mut html_output, parser);

    // Add cover image if exists
    let cover_image_html = get_cover_image_html(md_file, &config.images_dir);
    let final_html_output = format!(
        r#"
        <div class="post-container">
            {cover_image}
            <h2>{title}</h2>
            <div class="post-title-separator"></div>
            <span class="post-date">{date}</span>
            <div class="post-content">
                {post_content}
            </div>
        </div>
        "#,
        cover_image = cover_image_html,
        title = title,
        date = date,
        post_content = html_output,
    );
    final_html_output
}

// Helper function to move the image to the ./webpage/images/ directory
fn move_image_to_webpage(image_path: &str, images_dir: &str) {
    // let target_dir = "./webpage/images/";
    create_dir_all(images_dir).expect("Failed to create target directory");
    let image_filename = Path::new(image_path).file_name().unwrap().to_str().unwrap();
    let target_path = format!("{}{}", images_dir, image_filename);

    if !Path::new(&target_path).exists() {
        fs::copy(image_path, &target_path).expect("Failed to copy image file");
    }
}

// Helper function to get the cover image HTML
fn get_cover_image_html(md_file: &str, images_dir: &str) -> String {
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
fn get_thumbnail_meta_tag(md_file: &str, base_url: &String, images_dir: &str) -> String {
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

// Create HTML files from markdown files
fn create_html_files(config: &Config, md_files: Vec<String>) {
    println!("Creating HTML files");
    for md_file in md_files {
        let content = fs::read_to_string(&md_file).expect("Error reading file");
        let html_content = markdown_to_html(&content, &md_file, &config);
        let html_content = create_html_template(config, &html_content, false, &md_file);
        let html_file = md_file.replace(".md", ".html").replace("data/", "webpage/");
        fs::write(&html_file, html_content).expect("Error writing HTML file");
    }
}

// Create an index page
fn create_index_page(config: &Config, md_files: Vec<String>) {
    println!("Creating index page");
    let mut index_content = String::from("");
    let mut entries: Vec<(String, String, String)> = Vec::new();

    for md_file in &md_files {
        if md_file == "data/about.md" || md_file == "data/newsletter.md" {
            continue;
        }
        if config.is_prod && md_file == "data/example.md" {
            continue;
        }
        let article_name = md_file.replace(".md", ".html").replace("data/", "");
        let article_url = format!("{}/{}", config.base_url, article_name);
        let content = fs::read_to_string(&md_file).expect("Error reading file");
        let article_name = content.lines().next().unwrap_or("").replace("#", "");
        let date = content.lines().nth(1).unwrap_or("").to_string();
        entries.push((article_url, article_name, date));
    }

    entries.sort_by(|(_, _, a), (_, _, b)| {
        let date_a = NaiveDate::parse_from_str(a, "%m-%d-%Y")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let date_b = NaiveDate::parse_from_str(b, "%m-%d-%Y")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        date_b.cmp(&date_a)
    });

    for (article_url, article_name, date) in entries {
        index_content.push_str(&format!(
            r#"<div class='post'>
                <span class="index-date">{date}</span>
                <span class="index-post-title"><a href="{}">{}</a></span>
            </div>"#,
            article_url,
            article_name,
            date = date,
        ));
    }

    let index_content = create_html_template(config, &index_content, true, "");
    fs::write("./webpage/index.html", index_content).expect("Error writing index file");
}

// clear old HTML files
fn clear_html_files(webpage_dir: &str) {
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

fn create_html_template(config: &Config, content: &str, index: bool, md_file: &str) -> String {
    let container = if index {
        "index-container"
    } else {
        "container"
    };
    let thumbnail_meta_tag = if !index {
        get_thumbnail_meta_tag(md_file, &config.base_url, &config.images_dir)
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
            <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@300;500;700&display=swap" rel="stylesheet">
            <title>{title}</title>
            <link rel="stylesheet" type="text/css" href="{css_path}">
            <link rel="stylesheet" type="text/css" href="{mobile_css_path}">
            {thumbnail_meta_tag}
        </head>
        <body>
            <header>
                <nav>
                    <div class="nav-bar">
                        <div class="nav-item"> <h3><a href="{base_url}">Robert Miller</a></h3></div>
                        <div class="nav-item"> <a href="{base_url}">Writing</a></div>
                        <div class="nav-item"> <a href="{newsletter_url}">Newsletter</a></div>
                        <div class="nav-item"> <a href="{about_url}">About</a></div>
                    </div>
                </nav>
            </header>
            <div class="{container}">
                {content}
            </div>
        </body>
        </html>"#,
        title = config.title,
        css_path = config.css_path(),
        mobile_css_path = config.mobile_css_path(),
        content = content,
        base_url = format!("{}{}", config.base_url, "/index.html"),
        about_url = format!("{}{}", config.base_url, "/about.html"),
        newsletter_url = format!("{}{}", config.base_url, "/newsletter.html"),
        container = container,
        thumbnail_meta_tag = thumbnail_meta_tag,
    )
}
