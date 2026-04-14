use crate::config::Config;
use crate::core::FilteredList;
use crate::keybindings::Keybindings;
use crate::label::Label;
use std::collections::{BTreeMap, HashSet};
use std::io::{Read, Write};
use std::{fs, io};
use zellij_tile::prelude::*;

const CONFIGURATION_EXEC: &str = "exec";
const CONFIGURATION_SEPARATOR: &str = "separator";
const CONFIGURATION_CHROME_COLOR: &str = "chrome_color";
const CONFIGURATION_MATCH_COLOR: &str = "match_color";
const CONFIGURATION_ACTIVE_ITEM_COLOR: &str = "active_item_color";
const CONFIGURATION_SELECTED_ITEM_FRAME: &str = "selected_item_frame";
const CONFIGURATION_IGNORE_CASE: &str = "ignore_case";
const CONFIGURATION_FUZZY_SEARCH: &str = "fuzzy_search";
const CONFIGURATION_AUTODETECT_FILTER_MODE: &str = "autodetect_filter_mode";
const CONFIGURATION_FILENAME: &str = "filename";

use super::State;

impl State {
    fn create_config_if_not_exists(&self) -> io::Result<()> {
        let path = self.get_path();
        if !path.exists() {
            let conf: Config = Config::default();
            let serialized = serde_yaml::to_string(&conf).expect("Failed to serialize bookmarks.");
            let mut file = fs::File::create(&path)?;
            file.write_all(serialized.as_bytes())?;
        }
        Ok(())
    }

    pub(crate) fn load_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.get_path();

        let mut file = fs::File::open(&path)?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let config: Config = serde_yaml::from_str(&content)?;

        let mut set = HashSet::new();
        let mut labels = Vec::new();
        let mut label_id = 1;
        for bookmark in config.bookmarks.iter() {
            for label in bookmark.labels.iter() {
                if set.insert(label.clone()) {
                    labels.push(Label::new(label_id, label.clone()));
                    label_id += 1;
                };
            }
        }

        self.labels = FilteredList::new(labels);

        self.bookmarks = FilteredList::new(config.bookmarks.clone());

        self.config = config;

        Ok(())
    }

    pub(crate) fn load(&mut self, configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::WriteToStdin,
            PermissionType::OpenFiles,
        ]);

        if let Some(value) = configuration.get(CONFIGURATION_EXEC) {
            self.exec = value.trim().parse::<bool>().unwrap_or_else(|_| {
                self.error_mgr.handle_error(
                    format!("'{CONFIGURATION_EXEC}' config value must be 'true' or 'false', but it's '{value}'. The false is used.")
                );
                false
            })
        }

        if let Some(value) = configuration.get(CONFIGURATION_SEPARATOR) {
            self.separator = value.clone();
        }

        if let Some(value) = configuration.get(CONFIGURATION_SELECTED_ITEM_FRAME) {
            self.ui_style.selected_item_frame = value.trim().parse::<bool>().unwrap_or_else(|_| {
                self.error_mgr.handle_error(
                    format!("'{CONFIGURATION_SELECTED_ITEM_FRAME}' config value must be 'true' or 'false', but it's '{value}'. The true is used.")
                );
                true
            })
        }

        if let Some(value) = configuration.get(CONFIGURATION_CHROME_COLOR) {
            self.ui_style.chrome_color = value.trim().parse::<usize>().unwrap_or_else(|_| {
                self.error_mgr.handle_error(
                    format!("'{CONFIGURATION_CHROME_COLOR}' config value must be a number, but it's '{value}'. The 2 is used.")
                );
                2
            })
        }

        if let Some(value) = configuration.get(CONFIGURATION_MATCH_COLOR) {
            self.ui_style.match_color = value.trim().parse::<usize>().unwrap_or_else(|_| {
                self.error_mgr.handle_error(
                    format!("'{CONFIGURATION_MATCH_COLOR}' config value must be a number, but it's '{value}'. The 3 is used.")
                );
                3
            })
        }

        if let Some(value) = configuration.get(CONFIGURATION_ACTIVE_ITEM_COLOR) {
            self.ui_style.active_item_color =
                value.trim().parse::<usize>().unwrap_or_else(|_| {
                    self.error_mgr.handle_error(
                        format!("'{CONFIGURATION_ACTIVE_ITEM_COLOR}' config value must be a number, but it's '{value}'. The 0 is used.")
                    );
                    0
                })
        }

        if let Some(value) = configuration.get(CONFIGURATION_IGNORE_CASE) {
            self.ignore_case = value.trim().parse::<bool>().unwrap_or_else(|_| {
                self.error_mgr.handle_error(
                    format!("'{CONFIGURATION_IGNORE_CASE}' config value must be 'true' or 'false', but it's '{value}'. The true is used.")
                );
                true
            })
        }

        if let Some(value) = configuration.get(CONFIGURATION_FUZZY_SEARCH) {
            self.fuzzy_search = value.trim().parse::<bool>().unwrap_or_else(|_| {
                self.error_mgr.handle_error(
                    format!("'{CONFIGURATION_FUZZY_SEARCH}' config value must be 'true' or 'false', but it's '{value}'. The true is used.")
                );
                true
            })
        }

        if let Some(value) = configuration.get(CONFIGURATION_AUTODETECT_FILTER_MODE) {
            self.detect_filter_mode = value.trim().parse::<bool>().unwrap_or_else(|_| {
                self.error_mgr.handle_error(
                    format!("'{CONFIGURATION_AUTODETECT_FILTER_MODE}' config value must be 'true' or 'false', but it's '{value}'. The true is used.")
                );
                true
            })
        }

        if let Some(value) = configuration.get(CONFIGURATION_FILENAME) {
            if !value.is_empty() {
                self.filename = value.clone();
            }
        }

        if let Err(e) = self.create_config_if_not_exists() {
            self.error_mgr
                .handle_crit_error(format!("Failed to create file '{}': {}.", self.filename, e));
        }

        if let Err(e) = self.load_config() {
            self.error_mgr.handle_error(format!(
                "Failed to load config file '{}': {}.",
                self.filename, e
            ));
        }

        match Keybindings::new(configuration) {
            Ok(kb) => self.keybindings = kb,
            Err(e) => {
                self.error_mgr.handle_error(format!(
                    "Failed to parse zellij-bookmarks keybindings, check your config: {}. Default is used.", e
                ));
            }
        }

        subscribe(&[EventType::Key]);
    }
}
