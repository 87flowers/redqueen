use etcetera::{AppStrategy, AppStrategyArgs, choose_app_strategy};
use std::path::PathBuf;

pub struct Paths {
    config_dir: PathBuf,
    cache_dir: PathBuf,
}

impl Paths {
    pub fn new() -> Paths {
        let strategy = choose_app_strategy(AppStrategyArgs {
            top_level_domain: "87flowers.com".to_string(),
            author: "Red Queen".to_string(),
            app_name: "RQClient".to_string(),
        })
        .unwrap();

        Paths {
            config_dir: strategy.config_dir(),
            cache_dir: strategy.cache_dir(),
        }
    }

    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
}
