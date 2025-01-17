use std::path::PathBuf;

use crate::app::AppModel;
use crate::domain::program::Program;
use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    Application,
};

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    project_root_path: Option<PathBuf>,
    programs: Vec<Program>,
}

impl Config {
    pub fn load() -> (Option<cosmic_config::Config>, Self) {
        match cosmic_config::Config::new(AppModel::APP_ID, Config::VERSION) {
            Ok(config_handler) => {
                let config = match Config::get_entry(&config_handler) {
                    Ok(ok) => ok,
                    Err((errs, config)) => {
                        eprintln!("{:?}", errs);
                        config
                    }
                };
                (Some(config_handler), config)
            }
            Err(err) => {
                eprintln!("{:?}", err);
                (None, Config::default())
            }
        }
    }

    pub fn project_root_path(&self) -> Option<&PathBuf> {
        self.project_root_path.as_ref()
    }

    pub fn programs(&self) -> &[Program] {
        self.programs.as_slice()
    }
}
