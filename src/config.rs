use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_url")]
    pub url: String,

    #[serde(default = "default_frame_height")]
    pub frame_height: i32,
}

fn default_url() -> String {
    "127.0.0.1:8082".to_string()
}

fn default_frame_height() -> i32 {
    540
}

impl Default for Config {
    fn default() -> Self {
        Config {
            url: default_url(),
            frame_height: default_frame_height(),
        }
    }
}

pub fn load_config() -> Config {
    let config_path = Path::new("config.json");

    if config_path.exists() {
        match fs::read_to_string(config_path) {
            Ok(contents) => match serde_json::from_str::<Config>(&contents) {
                Ok(config) => {
                    println!("{}", "✓ Loaded config from config.json".green().bold());
                    return config;
                }
                Err(e) => {
                    eprintln!("{} {}", "⚠ Error parsing config.json:".yellow().bold(), e);
                    eprintln!("{}", "  Using defaults.".yellow());
                }
            },
            Err(e) => {
                eprintln!("{} {}", "⚠ Error reading config.json:".yellow().bold(), e);
                eprintln!("{}", "  Using defaults.".yellow());
            }
        }
    } else {
        println!("{}", "ℹ config.json not found. Using defaults.".cyan());
    }

    Config::default()
}
