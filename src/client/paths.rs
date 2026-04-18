use etcetera::{AppStrategy, AppStrategyArgs, choose_app_strategy};
use std::path::PathBuf;

use crate::common::domain::EngineBranch;

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

    pub fn config_file_path(&self) -> PathBuf {
        self.config_dir().join("rqclient.conf")
    }

    pub fn engine_executable(&self, branch: &EngineBranch) -> PathBuf {
        self.cache_dir().join("engines").join(format!("engine-{}-{}", branch.engine_id.0, branch.commit_hash))
    }
}
