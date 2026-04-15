use crate::config::Config;
use crate::core::FilteredList;
use crate::editable_file::EditableFile;
use crate::keybindings::Keybindings;
use crate::label::Label;
use std::collections::{BTreeMap, HashSet};
use std::io::{Read, Write};
use std::path::Path;
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
const CONFIGURATION_DIRNAME: &str = "dirname";

use super::State;

impl State {
    fn editable_files(&self) -> io::Result<Vec<EditableFile>> {
        let mut files = vec![EditableFile {
            id: 1,
            path: self.filename.clone(),
        }];

        for path in self.list_extra_config_paths()? {
            if let Ok(relative_path) = path.strip_prefix(self.get_cwd()) {
                files.push(EditableFile {
                    id: files.len() + 1,
                    path: relative_path.to_string_lossy().to_string(),
                });
            }
        }

        Ok(files)
    }

    pub(crate) fn refresh_editable_files(&mut self) -> io::Result<Vec<EditableFile>> {
        let files = self.editable_files()?;
        self.editable_files = FilteredList::new(files.clone());
        Ok(files)
    }

    fn create_config_if_not_exists(&self) -> io::Result<()> {
        let path = self.get_path();
        if !path.exists() {
            let conf: Config = Config::default();
            let serialized = serde_yaml::to_string(&conf).expect("Failed to serialize bookmarks.");
            let mut file = fs::File::create(&path)?;
            file.write_all(serialized.as_bytes())?;
        }

        let dir_path = self.get_dir_path();
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)?;
        }

        Ok(())
    }

    fn read_config(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
        let mut file = fs::File::open(path)?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(serde_yaml::from_str(&content)?)
    }

    fn read_file_config(&self, file: &EditableFile) -> Result<Config, Box<dyn std::error::Error>> {
        let mut config = Self::read_config(&self.editable_file_path(file))?;
        let managed_label = file.managed_label(&self.filename, &self.dirname);

        for bookmark in &mut config.bookmarks {
            bookmark.add_managed_label(managed_label.clone());
        }

        Ok(config)
    }

    fn editable_file_path(&self, file: &EditableFile) -> std::path::PathBuf {
        self.get_cwd().join(file.path.as_str())
    }

    fn list_extra_config_paths(&self) -> io::Result<Vec<std::path::PathBuf>> {
        let dir_path = self.get_dir_path();
        if !dir_path.exists() {
            return Ok(Vec::new());
        }

        let mut extra_files = fs::read_dir(&dir_path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        extra_files.retain(|path| path.is_file() && Self::is_yaml_file(path));
        extra_files.sort();

        Ok(extra_files)
    }

    fn is_yaml_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "yaml" | "yml"))
            .unwrap_or(false)
    }

    pub(crate) fn load_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let files = self.refresh_editable_files().map_err(io::Error::other)?;
        let mut files_iter = files.iter();
        let Some(first_file) = files_iter.next() else {
            return Err(io::Error::other("No editable files found").into());
        };

        let mut config = self.read_file_config(first_file)?;
        let mut merged_file = first_file;

        for file in files_iter {
            let file_config = self.read_file_config(file)?;
            config.merge(file_config).map_err(|err| {
                io::Error::other(format!(
                    "Failed to merge config files '{}' and '{}': {}",
                    merged_file.path, file.path, err
                ))
            })?;
            merged_file = file;
        }

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

        if let Some(value) = configuration.get(CONFIGURATION_DIRNAME) {
            if !value.is_empty() {
                self.dirname = value.clone();
            }
        }

        if let Err(e) = self.create_config_if_not_exists() {
            self.error_mgr.handle_crit_error(format!(
                "Failed to initialize config storage '{}', '{}': {}.",
                self.filename, self.dirname, e
            ));
        }

        if let Err(e) = self.load_config() {
            self.error_mgr.handle_error(format!(
                "Failed to load config from main file '{}' and extra config dir '{}': {}.",
                self.filename, self.dirname, e
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
