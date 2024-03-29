use serde::Deserialize;
use std::fs;
use std::str::FromStr;

#[derive(Deserialize, Default, Clone)]
pub struct Config {
    pub overlay: OverlayConfig,
    pub watcher: WatcherConfig,
}

#[derive(Deserialize, Default, Clone)]
pub struct OverlayConfig {
    pub x_offset: u32,
    pub y_offset: u32,
    pub show_pattern: Vec<i32>,
}

#[derive(Deserialize, Default, Clone)]
pub struct WatcherConfig {
    pub poll_frequency: u64,
    pub notification_frequency: u64,
    pub ignore: Vec<String>,
}

pub fn load() -> Result<Config, toml::de::Error> {
    #[cfg(not(test))]
    {
        let mut dir = std::env::current_exe().unwrap();
        dir.pop();

        // TODO ignore patterns should support wildcards, not regexp

        let p = format!("{}\\config.toml", dir.to_str().unwrap());

        return toml::from_str(fs::read_to_string(p).unwrap().as_str());
    }
    #[cfg(test)]
    {
        Ok(Config::default())
    }
}
