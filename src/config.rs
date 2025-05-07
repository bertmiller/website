use dotenv::dotenv;
use std::env;
use tracing::info;

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
const DEFAULT_TITLE: &str = "Title";

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let args: Vec<String> = env::args().collect();
        let is_prod = args.contains(&"--prod".to_string());
        let current_dir = env::current_dir()?.to_string_lossy().to_string();
        let title = env::var("TITLE").unwrap_or_else(|_| DEFAULT_TITLE.to_string());

        let base_url = if is_prod {
            env::var("BASE_URL").unwrap_or_else(|_| format!("{}/webpage", &current_dir))
        } else {
            format!("{}/webpage", &current_dir)
        };

        let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| DEFAULT_DATA_DIR.to_string());
        let webpage_dir =
            env::var("WEBPAGE_DIR").unwrap_or_else(|_| DEFAULT_WEB_PAGE_DIR.to_string());
        let images_dir = env::var("IMAGES_DIR")
            .unwrap_or_else(|_| format!("{}{}", webpage_dir, DEFAULT_IMAGES_DIR));

        info!("Using base_url: {}", base_url);
        info!("Using title: {}", title);
        info!("Using data_dir: {}", data_dir);
        info!("Using webpage_dir: {}", webpage_dir);
        info!("Using images_dir: {}", images_dir);

        Ok(Config {
            is_prod,
            base_url,
            title,
            data_dir,
            webpage_dir,
            images_dir,
        })
    }

    pub fn css_path(&self) -> String {
        format!("{}/main.css", self.base_url)
    }

    pub fn mobile_css_path(&self) -> &'static str {
        if self.is_prod {
            "./mobile.css"
        } else {
            "../webpage/mobile.css"
        }
    }
}
