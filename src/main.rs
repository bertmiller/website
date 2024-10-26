use glob::glob;
use pulldown_cmark::{Parser, html::push_html, Options};
use notify::{Watcher, RecursiveMode, Result as notifyResult};
use std::{
    fs, env,
    sync::mpsc::channel,
    path::Path,
};
use std::fs::create_dir_all;
use chrono::NaiveDate;
use dotenv::dotenv;

fn create_files(md_files: Vec<String>, is_prod: bool, base_url: String, title: String){
    clear_html_files();
    create_html_files(md_files.clone(), is_prod, base_url.clone(), title.clone());
    create_index_page(md_files, base_url, title, is_prod);
}

fn main() -> notifyResult<()> {
    dotenv().ok();
    let args: Vec<String> = env::args().collect();
    let is_prod = args.contains(&"--prod".to_string());
    let current_dir = env::current_dir().expect("Failed to get current directory").to_string_lossy().to_string();
    let title = env::var("TITLE").unwrap_or_else(|_| "Title".to_string());

    let base_url = if is_prod {
        env::var("BASE_URL").unwrap_or_else(|_| format!("{}/webpage", &current_dir))
    } else {
        format!("{}/webpage",&current_dir)
    };

    println!("Using base_url: {}", base_url);
    println!("Using title: {}", title);

    let md_files = read_markdown_files("./data/");
    create_files(md_files.clone(), is_prod, base_url.clone(), title.clone());
    
    let (_tx, rx) = channel::<String>();

    let mut watcher = notify::recommended_watcher(move |res| {
        match res {
            Ok(event) => {
                println!("{:?}", event);
                create_files(md_files.clone(), is_prod, base_url.clone(), title.clone());
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    })?;

    watcher.watch(Path::new("./data/"), RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(_) => {},
            Err(e) => println!("watch receive error: {:?}", e),
        }
    }
}

// Read markdown files from a folder
fn read_markdown_files(folder: &str) -> Vec<String> {
    let pattern = format!("{}/*.md", folder);
    glob(&pattern).expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .map(|path| path.display().to_string())
        .collect()
}

// Convert markdown to HTML
fn markdown_to_html(markdown: &str, md_file: &str) -> String {
    // Extract the title (first h2) and date (second line)
    let mut lines = markdown.lines();
    let title = lines.next().unwrap_or("").trim_start_matches("## ").to_string();
    let date = lines.next().unwrap_or("").to_string();

    // Skip any empty lines after the date
    let content = lines.skip_while(|line| line.trim().is_empty())
                       .collect::<Vec<&str>>()
                       .join("\n");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&content, options);
    let mut html_output = String::new();
    push_html(&mut html_output, parser);

    // Add cover image if exists
    let cover_image_html = get_cover_image_html(md_file);
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
fn move_image_to_webpage(image_path: &str) {
    let target_dir = "./webpage/images/";
    create_dir_all(target_dir).expect("Failed to create target directory");
    let image_filename = Path::new(image_path).file_name().unwrap().to_str().unwrap();
    let target_path = format!("{}{}", target_dir, image_filename);

    if !Path::new(&target_path).exists() {
        fs::copy(image_path, &target_path).expect("Failed to copy image file");
    }
}

// Helper function to get the cover image HTML
fn get_cover_image_html(md_file: &str) -> String {
    let base_filename = Path::new(md_file).file_stem().unwrap().to_str().unwrap();
    let extensions = ["jpg", "jpeg", "png", "gif"];
    for ext in &extensions {
        let image_path = format!("./images/{}.{}", base_filename, ext);
        if Path::new(&image_path).exists() {
            move_image_to_webpage(&image_path);
            return format!(r#"<img class="cover-image" src="{}" alt="Cover Image">"#, image_path);
        }
    }
    String::new()
}

// Helper function to get the thumbnail meta tag and move the image
fn get_thumbnail_meta_tag(md_file: &str, base_url: &String) -> String {
    let base_filename = Path::new(md_file).file_stem().unwrap().to_str().unwrap();
    let extensions = ["jpg", "jpeg", "png", "gif"];
    for ext in &extensions {
        let image_path = format!("./images/{}.{}", base_filename, ext);
        if Path::new(&image_path).exists() {
            move_image_to_webpage(&image_path);
            let thumbnail_url = format!("{}/{}", base_url, image_path.replace("./",""));
            return format!(r#"<meta property="og:image" content="{}"/>"#, thumbnail_url);
        }
    }
    String::new()
}

// Create HTML files from markdown files
fn create_html_files(md_files: Vec<String>, is_prod: bool, base_url: String, title: String) {
    println!("Creating HTML files");
    let css_path = if is_prod {
        "./main.css"
    } else {
        "../webpage/main.css"
    };
    for md_file in md_files {
        let content = fs::read_to_string(&md_file).expect("Error reading file");
        let html_content = markdown_to_html(&content, &md_file);
        let html_content = create_html_template(css_path, &html_content, base_url.clone(), title.clone(), false, &md_file);
        let html_file = md_file.replace(".md", ".html");
        let html_file = html_file.replace("data/", "webpage/");
        fs::write(&html_file, html_content).expect("Error writing HTML file");
    }
}

// Create an index page
fn create_index_page(md_files: Vec<String>, base_url: String, title: String, is_prod: bool) {
    println!("Creating index page");
    let mut index_content = String::from("");
    let mut entries: Vec<(String, String, String)> = Vec::new();

    for md_file in &md_files {
        if md_file == "data/about.md" || md_file == "data/newsletter.md" {
            continue;
        }
        if is_prod && md_file == "data/example.md" {
            continue;
        }
        let article_name = md_file.replace(".md", ".html")
                                  .replace("data/", "");
        let article_url = format!("{}/{}", base_url, article_name);
        let content = fs::read_to_string(&md_file).expect("Error reading file");
        let article_name = content.lines()
                                    .next()
                                    .unwrap_or("")
                                    .replace("#", "");
        let date = content.lines()
                            .nth(1)
                            .unwrap_or("")
                            .to_string();
        entries.push((article_url, article_name, date));
    }

    entries.sort_by(|(_, _, a), (_, _, b)| {
        let date_a = NaiveDate::parse_from_str(a, "%m-%d-%Y").unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let date_b = NaiveDate::parse_from_str(b, "%m-%d-%Y").unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
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

    index_content = create_html_template("./main.css", &index_content, base_url, title, true, "");
    fs::write("./webpage/index.html", index_content).expect("Error writing index file");
}

// clear old HTML files
fn clear_html_files() {
    println!("Clearing old files");
    let pattern = "./webpage/*.html";
    for entry in glob(pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                fs::remove_file(path).expect("Error removing file");
            },
            Err(e) => println!("{:?}", e),
        }
    }
}

fn create_html_template(css_path: &str, content: &str, base_url: String, title: String, index: bool, md_file: &str) -> String {
    let container = if index {
        "index-container"
    } else {
        "container"
    };
    let thumbnail_meta_tag = if !index {
        get_thumbnail_meta_tag(md_file, &base_url)
    } else {
        String::new()
    };
    format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
            <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@300;500;700&display=swap" rel="stylesheet">
            <title>{title}</title>
            <link rel="stylesheet" type="text/css" href="{css_path}">
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
        title = title,
        css_path = css_path,
        content = content,
        base_url = format!("{}{}",base_url, "/index.html"),
        about_url = format!("{}{}",base_url, "/about.html"),
        newsletter_url = format!("{}{}",base_url, "/newsletter.html"),
        container = container,
        thumbnail_meta_tag = thumbnail_meta_tag,
    )
}
