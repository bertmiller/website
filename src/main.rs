use glob::glob;
use pulldown_cmark::{Parser, html::push_html, Options};
use std::fs;
use notify::{Watcher, RecursiveMode, Result as notifyResult};
use std::sync::mpsc::channel;
use std::path::Path;
use chrono::NaiveDate;

fn main() -> notifyResult<()> {
    let (_tx, rx) = channel::<String>();

    let mut watcher = notify::recommended_watcher(move |res| {
        match res {
            Ok(event) => {
                println!("{:?}", event);
                clear_html_files();
                let md_files = read_markdown_files("./data/");
                create_html_files(md_files.clone());
                create_index_page(md_files);
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
fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::empty());
    let mut html_output = String::new();
    push_html(&mut html_output, parser);
    html_output
}

// Create HTML files from markdown files
fn create_html_files(md_files: Vec<String>) {
    for md_file in md_files {
        let content = fs::read_to_string(&md_file).expect("Error reading file");
        let html_content = markdown_to_html(&content);
        let html_content = create_html_template("../webpage/main.css", &html_content);
        let html_file = md_file.replace(".md", ".html");
        let html_file = html_file.replace("data/", "webpage/");
        println!("Creating file: {}", html_file);
        fs::write(&html_file, html_content).expect("Error writing HTML file");
    }
}

// Create an index page
fn create_index_page(md_files: Vec<String>) {
    let mut index_content = String::from("<ul>");
    let mut entries: Vec<(String, String, String)> = Vec::new();

    for md_file in &md_files {
        let file_name = md_file.replace(".md", ".html");
        let file_name = file_name.replace("data/", "");
        let content = fs::read_to_string(&md_file).expect("Error reading file");
        let first_line = content.lines().next().unwrap_or("");
        let first_line = first_line.replace("#", "");
        let second_line = content.lines().nth(1).unwrap_or("");
        println!("Creating index entry: {}", first_line);
        println!("Date: {}", second_line);
        entries.push((file_name, first_line, second_line.to_string()));
    }

    entries.sort_by(|(_, _, a), (_, _, b)| {
        println!("A: {}", a);
        println!("B: {}", b);
        let date_a = NaiveDate::parse_from_str(a, "%m-%d-%Y").unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1));
        let date_b = NaiveDate::parse_from_str(b, "%m-%d-%Y").unwrap_or_else(|_| NaiveDate::from_ymd(1970, 1, 1));
        println!("Date A: {}", date_a);
        println!("Date B: {}", date_b);
        println!("Date A cmp Date B: {:?}", date_a.cmp(&date_b));
        date_b.cmp(&date_a)
    });

    for (file_name, entry, date) in entries {
        println!("file_name: {}", file_name);   
        println!("Entry: {}", entry);
        println!("Date: {}", date);
        index_content.push_str(&format!("<li><a href=\"{}\">{}</a> - {}</li>", file_name, entry, date));
    }

    index_content.push_str("</ul>");
    index_content = create_html_template("./main.css", &index_content);
    fs::write("./webpage/index.html", index_content).expect("Error writing index file");
}

// clear old HTML files
fn clear_html_files() {
    let pattern = "./webpage/*.html";
    for entry in glob(pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                println!("Removing file: {}", path.display());
                fs::remove_file(path).expect("Error removing file");
            },
            Err(e) => println!("{:?}", e),
        }
    }
}

fn create_html_template(css_path: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
            <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@300;500;700&display=swap" rel="stylesheet">
            <title>Robert Miller</title>
            <link rel="stylesheet" type="text/css" href="{css_path}">
        </head>
        <body>
            <header>
                <nav>
                    <div class="nav-bar">
                        <div class="nav-item"> <h3><a href="/">Robert Miller</a></h3> </div>
                    </div>
                </nav>
            </header>
            <div class="container">
                {content}
            </div>
        </body>
        </html>"#,
        css_path = css_path,
        content = content
    )
}