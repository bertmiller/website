use dotenv::dotenv;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub is_prod: bool,
    pub base_url: String,
    pub title: String,
    pub data_dir: String,
    pub webpage_dir: String,
    pub images_dir: String,
}

const DEFAULT_DATA_DIR: &str = "./data/";
const DEFAULT_WEB_PAGE_DIR: &str = "./webpage/";
const DEFAULT_IMAGES_DIR: &str = "images/";

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let args: Vec<String> = env::args().collect();
        let is_prod = args.contains(&"--prod".to_string());
        let current_dir = env::current_dir()?.to_string_lossy().to_string();
        let title = env::var("TITLE").unwrap_or_else(|_| "Title".to_string());

        let base_url = if is_prod {
            env::var("BASE_URL").unwrap_or_else(|_| format!("{}/webpage", &current_dir))
        } else {
            format!("{}/webpage", &current_dir)
        };

        println!("Using base_url: {}", base_url);
        println!("Using title: {}", title);

        Ok(Config {
            is_prod,
            base_url,
            title,
            data_dir: DEFAULT_DATA_DIR.into(),
            webpage_dir: DEFAULT_WEB_PAGE_DIR.into(),
            images_dir: format!("{}{}", DEFAULT_WEB_PAGE_DIR, DEFAULT_IMAGES_DIR),
        })
    }

    pub fn css_path(&self) -> &'static str {
        if self.is_prod {
            "./main.css"
        } else {
            "../webpage/main.css"
        }
    }

    pub fn mobile_css_path(&self) -> &'static str {
        if self.is_prod {
            "./mobile.css"
        } else {
            "../webpage/mobile.css"
        }
    }
}
