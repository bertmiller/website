use crate::config::Config;
use crate::file_utils;
use chrono::NaiveDate;
use pulldown_cmark::{html::push_html, Options, Parser};
use std::fs;
use std::path::Path;
use tracing::info;

// Estimate reading time based on word count.
// Assumes an average reading speed of 200 words per minute.
fn estimate_reading_time(text: &str) -> u32 {
    let word_count = text.split_whitespace().count();
    let wpm = 200;
    let minutes = (word_count as f64 / wpm as f64).ceil() as u32;
    if minutes < 1 {
        1
    } else {
        minutes
    }
}

// Convert markdown to HTML
pub fn markdown_to_html(markdown: &str, md_file: &str, config: &Config) -> String {
    // Extract the title (first h2) and date (second line)
    let mut lines = markdown.lines();
    let title = lines
        .next()
        .unwrap_or("")
        .trim_start_matches("## ")
        .to_string();
    let date_line = lines.next().unwrap_or("").to_string();
    let mut substack_link_html = String::new();

    // Check for [substack post] on the third line (with a space)
    if let Some(third_line) = lines.next() {
        if third_line.starts_with("[substack post]") {
            let url = third_line.trim_start_matches("[substack post]").trim();
            // now remove the ()
            let url = url.trim_start_matches("(").trim_end_matches(")");
            if !url.is_empty() {
                substack_link_html =
                    format!(r#" / <a href="{}" target="_blank">Substack</a>"#, url);
            }
        }
    }

    let mut content_lines = markdown.lines();
    content_lines.next(); // Skip title
    content_lines.next(); // Skip date line
    if markdown
        .lines()
        .nth(2)
        .map_or(false, |l| l.starts_with("[substack post]"))
    {
        content_lines.next();
    }

    // Skip any empty lines after the header lines
    let content = content_lines
        .skip_while(|line| line.trim().is_empty())
        .collect::<Vec<&str>>()
        .join("\n");

    // Estimate reading time based on content
    let reading_time = estimate_reading_time(&content);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&content, options);
    let mut html_output = String::new();
    push_html(&mut html_output, parser);

    // Add cover image if exists
    let cover_image_html = file_utils::get_cover_image_html(md_file, config);
    let final_html_output = format!(
        r#"
        <div class="post-container">
            <h2>{title}</h2>
            <div class="post-title-separator"></div>
            <span class="post-date">{date} / {reading_time} min read {substack_link}</span>
            {cover_image}
            <div class="post-content">
                {post_content}
            </div>
        </div>
        "#,
        cover_image = cover_image_html,
        title = title,
        date = date_line,
        substack_link = substack_link_html,
        reading_time = reading_time,
        post_content = html_output,
    );
    final_html_output
}

// Create HTML files from markdown files
pub fn create_blog_posts(config: &Config, md_files: Vec<String>) {
    info!("Creating HTML files");
    for md_file in md_files {
        let content = fs::read_to_string(&md_file).expect("Error reading file");
        let html_content = markdown_to_html(&content, &md_file, config);
        let html_content = create_html_template(config, &html_content, false, &md_file);

        // Extract the file name from the md_file path
        let file_name = Path::new(&md_file)
            .file_name()
            .and_then(|n| n.to_str())
            .expect("Failed to extract file name");

        // Construct the new HTML file path
        let html_file = format!(
            "{}{}.html",
            config.webpage_dir,
            file_name.trim_end_matches(".md")
        );

        fs::write(&html_file, html_content).expect("Error writing HTML file");
    }
}

// Create an index page
pub fn create_index_page(config: &Config, md_files: Vec<String>) {
    info!("Creating index page");
    let mut index_content = String::new();
    let mut entries: Vec<(String, String, String)> = Vec::new();

    for md_file in &md_files {
        let about_path = format!("{}about.md", config.data_dir);
        let newsletter_path = format!("{}newsletter.md", config.data_dir);
        let example_path = format!("{}example.md", config.data_dir);
        let not_found_path = format!("{}404.md", config.data_dir);

        // Remove "./" from the beginning of the paths if present
        let md_file = md_file.strip_prefix("./").unwrap_or(md_file);
        let about_path = about_path.strip_prefix("./").unwrap_or(&about_path);
        let newsletter_path = newsletter_path
            .strip_prefix("./")
            .unwrap_or(&newsletter_path);
        let example_path = example_path.strip_prefix("./").unwrap_or(&example_path);
        let not_found_path = not_found_path.strip_prefix("./").unwrap_or(&not_found_path);
        if md_file == about_path || md_file == newsletter_path || md_file == not_found_path {
            continue;
        }
        if config.is_prod && md_file == example_path {
            continue;
        }

        let article_name = md_file.replace(".md", ".html").replace("data/", "");
        let article_url = if config.is_prod {
            format!("/{}", article_name)
        } else {
            format!("{}/{}", config.base_url, article_name)
        };
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
    let index_file_path = format!("{}/index.html", config.webpage_dir);
    fs::write(&index_file_path, index_content).expect("Error writing index file");
}

fn create_html_template(config: &Config, content: &str, index: bool, md_file: &str) -> String {
    let container = if index {
        "index-container"
    } else {
        "container"
    };
    let thumbnail_meta_tag = if !index {
        file_utils::get_thumbnail_meta_tag(md_file, config)
    } else {
        String::new()
    };

    let about_path = format!("{}about.md", config.data_dir);
    let about_class_contents = if md_file == (about_path).strip_prefix("./").unwrap_or(&about_path)
    {
        "class = about"
    } else {
        ""
    };

    let newsletter_path = format!("{}newsletter.md", config.data_dir);
    let newsletter_class_contents = if md_file
        == (newsletter_path)
            .strip_prefix("./")
            .unwrap_or(&newsletter_path)
    {
        "class = newsletter"
    } else {
        ""
    };

    let body_class = if index { "class = no-progress-bar" } else { "" };

    let footer_html = if index {
        String::new()
    } else {
        format!(
            r#"<footer>
                <div class="footer-content">
                    <div class="footer-line"></div>
                    <p class="footer-text">&copy; {} {}</p>
                </div>
            </footer>"#,
            chrono::Utc::now().format("%Y"), // Get current year
            "Robert Miller"                  // Your name or site name
        )
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
        <body {body_class}>
            <header>
                <nav>
                    <div class="nav-bar">
                        <div class="nav-item"> <h3><a href="{base_url}" {extra_class}>Robert Miller</a></h3></div>
                        <div class="nav-item"> <a href="{newsletter_url}" {newsletter_class}>Newsletter</a></div>
                        <div class="nav-item"> <a href="{about_url}" {about_class}>About</a></div>
                    </div>
                </nav>
            </header>
            <div class="{container}">
                {content}
            </div>
            {footer}
        </body>
        </html>"#,
        title = config.title,
        css_path = config.css_path(),
        mobile_css_path = config.mobile_css_path(),
        content = content,
        base_url = format!("{}", config.base_url),
        extra_class = "",
        about_url = format!("{}{}", config.base_url, "/about.html"),
        about_class = about_class_contents,
        newsletter_url = format!("{}{}", config.base_url, "/newsletter.html"),
        newsletter_class = newsletter_class_contents,
        container = container,
        thumbnail_meta_tag = thumbnail_meta_tag,
        footer = footer_html,
    )
}
