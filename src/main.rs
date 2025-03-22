mod filters;
mod keybindings;
mod render;
mod tab_manager;

use crate::filters::{BookmarkFilter, Filter, LabelFilter};
use crate::keybindings::{Keybinding, Keybindings};
use crate::tab_manager::TabManager;

use handlebars::Handlebars;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::io::{Read, Write};
use std::{fmt, io};
use std::{fs, path};
use zellij_tile::prelude::*;

const CONFIGURATION_EXEC: &str = "exec";
const CONFIGURATION_IGNORE_CASE: &str = "ignore_case";
const CONFIGURATION_AUTODETECT_FILTER_MODE: &str = "autodetect_filter_mode";
const CONFIGURATION_FILENAME: &str = "filename";

const CWD: &str = "/host";

const BASE_COLOR: usize = 2;

const RESERVE_ROW_COUNT: usize = 5;
const RESERVE_COLUMN_COUNT: usize = 12;

type Bookmarks = Vec<Bookmark>;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct Bookmark {
    #[serde(default)]
    id: usize,
    name: String,
    #[serde(default)]
    desc: String,
    cmds: Vec<String>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    vars: HashMap<String, String>,
    exec: Option<bool>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct Config {
    #[serde(default)]
    vars: HashMap<String, String>,
    #[serde(default)]
    cmds: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_bookmarks")]
    bookmarks: Bookmarks,
}

#[derive(Default, Debug, Clone)]
struct Label {
    id: usize,
    name: String,
}

impl Label {
    fn new(id: usize, name: String) -> Self {
        Self { id, name }
    }
}

fn deserialize_bookmarks<'de, D>(deserializer: D) -> Result<Bookmarks, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let bookmarks: Vec<Bookmark> = Vec::deserialize(deserializer)?;
    let mut result: Vec<Bookmark> = Vec::new();
    let mut uniq: HashSet<String> = HashSet::new();
    let mut duplicate_count = 0;

    for (i, mut bookmark) in bookmarks.into_iter().enumerate() {
        if !uniq.insert(bookmark.name.clone()) {
            duplicate_count += 1;
        } else {
            bookmark.id = i + 1;
            result.push(bookmark);
        }
    }

    if duplicate_count > 0 {
        return Err(serde::de::Error::custom(format!(
            "Duplicate bookmarks names: {}",
            duplicate_count
        )));
    }

    Ok(result)
}

struct State {
    mode: Mode,
    exec: bool,
    ignore_case: bool,
    detect_filter_mode: bool,
    filename: String,
    filter_mode: filters::Mode,
    filter: String,
    bookmarks_mgr: TabManager<Bookmark>,
    labels_mgr: TabManager<Label>,
    config: Config,
    keybindings: Keybindings,
    view_desc: bool,
    crit_error_message: Option<String>,
    error_message: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            exec: false,
            ignore_case: true,
            detect_filter_mode: true,
            filename: ".zellij_bookmarks.yaml".to_string(),
            filter_mode: Default::default(),
            filter: "".to_string(),
            bookmarks_mgr: Default::default(),
            labels_mgr: Default::default(),
            config: Default::default(),
            keybindings: Default::default(),
            view_desc: false,
            crit_error_message: None,
            error_message: None,
        }
    }
}

#[derive(Default, PartialEq, Debug, TryFromPrimitive, IntoPrimitive, Clone, Copy)]
#[repr(u32)]
enum Mode {
    #[default]
    Bookmarks = 1,
    Labels = 2,
    Usage = 3,
}

trait Navigation {
    fn next(&self) -> Self;
    fn prev(&self) -> Self;
    fn iter() -> impl Iterator<Item = Self>;
}

impl Navigation for Mode {
    fn next(&self) -> Mode {
        let next = (*self as u32).saturating_add(1);
        Mode::try_from(next).unwrap_or(Mode::Bookmarks)
    }

    fn prev(&self) -> Mode {
        let prev = (*self as u32).saturating_sub(1);
        Mode::try_from(prev).unwrap_or(Mode::Usage)
    }

    fn iter() -> impl Iterator<Item = Self> {
        (1..=3).filter_map(|v| Mode::try_from(v).ok())
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Bookmarks => "Bookmarks",
            Self::Labels => "Labels",
            Self::Usage => "Usage",
        };
        write!(f, "{}", name)
    }
}

impl State {
    fn bookmark_filter(&self) -> Box<dyn Filter<Bookmark>> {
        Box::new(BookmarkFilter::new(
            self.filter_mode,
            self.filter.clone(),
            self.ignore_case,
        ))
    }

    fn label_filter(&self) -> Box<dyn Filter<Label>> {
        Box::new(LabelFilter::new(
            self.filter_mode,
            self.filter.clone(),
            self.ignore_case,
        ))
    }

    fn set_filter(&mut self) {
        match self.mode {
            Mode::Bookmarks => self.bookmarks_mgr.with_filter(self.bookmark_filter()),
            Mode::Labels => self.labels_mgr.with_filter(self.label_filter()),
            _ => {}
        }
    }

    fn reset_selection(&mut self) {
        self.bookmarks_mgr.reset_selection();
        self.labels_mgr.reset_selection();
    }

    fn get_cwd(&self) -> path::PathBuf {
        path::PathBuf::from(CWD)
    }

    fn get_path(&self) -> path::PathBuf {
        self.get_cwd().join(self.filename.as_str())
    }

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

    fn load_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

        self.labels_mgr = TabManager::new(labels);

        self.bookmarks_mgr = TabManager::new(config.bookmarks.clone());

        self.config = config;

        Ok(())
    }

    fn handle_error(&mut self, error_message: String) {
        self.error_message = Some(error_message.clone());
        eprintln!("Error: {}", error_message);
    }

    fn handle_crit_error(&mut self, error_message: String) {
        self.crit_error_message = Some(error_message.clone());
        eprintln!("Critical Error: {}", error_message);
    }

    fn render_errors(&mut self) -> bool {
        if let Some(e) = self.crit_error_message.as_ref() {
            let text = Text::new(format!("ERROR: {}", e.red()));
            print_text_with_coordinates(text, 1, 1, None, None);
            return true;
        }
        if let Some(e) = self.error_message.take() {
            let text = Text::new(format!("ERROR: {}", e.red()));
            print_text_with_coordinates(text, 1, 1, None, None);
            return true;
        }
        false
    }

    fn render_usage(&self) {
        render::render_mode(2, 0, Mode::Usage);

        let mut table = Table::new();

        table = table.add_row(vec!["KeyBinding", "Action", "Mode", "Configurable"]);
        // Non configurable
        table = table.add_row(vec![
            format!(
                "{}|{}",
                BareKey::Esc,
                Keybinding::new(KeyModifier::Ctrl, 'c')
            )
            .as_str(),
            "Exit the zellij-bookmarks.",
            "*",
            "False",
        ]);
        table = table.add_row(vec![
            format!("{}|{} {}", BareKey::Tab, BareKey::Down, BareKey::Up).as_str(),
            "Navigate through the list of bookmarks or labels.",
            format!("{}|{}", Mode::Bookmarks, Mode::Labels).as_str(),
            "False",
        ]);
        table = table.add_row(vec![
            format!("{} {}", BareKey::Left, BareKey::Right).as_str(),
            "Switch between modes.",
            "*",
            "False",
        ]);
        table = table.add_row(vec![
            BareKey::Backspace.to_string().as_str(),
            "Remove the last character from the filter.",
            format!("{}|{}", Mode::Bookmarks, Mode::Labels).as_str(),
            "False",
        ]);
        table = table.add_row(vec![
            BareKey::Enter.to_string().as_str(),
            "Paste the selected bookmark into the terminal.",
            Mode::Bookmarks.to_string().as_str(),
            "False",
        ]);
        table = table.add_row(vec![
            BareKey::Enter.to_string().as_str(),
            "Find all bookmarks associated with the selected label.",
            Mode::Labels.to_string().as_str(),
            "False",
        ]);
        table = table.add_row(vec![
            format!("{:?} {}", KeyModifier::Ctrl, Mode::Bookmarks as u32).as_str(),
            "Switch to Bookmarks mode.",
            "*",
            "False",
        ]);
        table = table.add_row(vec![
            format!("{:?} {}", KeyModifier::Ctrl, Mode::Labels as u32).as_str(),
            "Switch to Labels mode.",
            "*",
            "False",
        ]);
        table = table.add_row(vec![
            format!("{:?} {}", KeyModifier::Ctrl, Mode::Usage as u32).as_str(),
            "Switch to Usage mode to view instructions.",
            "*",
            "False",
        ]);

        // Configurable
        table = table.add_row(vec![
            self.keybindings.edit.to_string().as_str(),
            "Open the bookmark configuration file in an editor.",
            "*",
            "True",
        ]);
        table = table.add_row(vec![
            self.keybindings.reload.to_string().as_str(),
            "Reload bookmarks. Required after modifying the configuration file.",
            "*",
            "True",
        ]);
        table = table.add_row(vec![
            self.keybindings.switch_filter_label.to_string().as_str(),
            "Switch to label filtering mode.",
            Mode::Bookmarks.to_string().as_str(),
            "True",
        ]);
        table = table.add_row(vec![
            self.keybindings.switch_filter_id.to_string().as_str(),
            "Switch to id filtering mode.",
            format!("{}|{}", Mode::Bookmarks, Mode::Labels).as_str(),
            "True",
        ]);
        table = table.add_row(vec![
            self.keybindings.describe.to_string().as_str(),
            "Show the description of the selected bookmark.",
            Mode::Bookmarks.to_string().as_str(),
            "True",
        ]);

        print_table_with_coordinates(table, 2, 2, None, None);
    }

    fn render_labels(&self, rows: usize, cols: usize) {
        let iter = self.labels_mgr.iter().map(|(i, l)| (i, l.id, &l.name));
        render::render_main_menu(
            rows,
            cols,
            self.labels_mgr.get_position(),
            self.labels_mgr.len(),
            Mode::Labels,
            self.filter.clone(),
            self.filter_mode.to_string(),
            iter,
        );
    }

    fn render_bookmarks(&self, rows: usize, cols: usize) {
        let iter = self.bookmarks_mgr.iter().map(|(i, b)| {
            if self.view_desc {
                (i, b.id, &b.desc)
            } else {
                (i, b.id, &b.name)
            }
        });
        render::render_main_menu(
            rows,
            cols,
            self.bookmarks_mgr.get_position(),
            self.bookmarks_mgr.len(),
            Mode::Bookmarks,
            self.filter.clone(),
            self.filter_mode.to_string(),
            iter,
        );
    }

    fn gen_template_command(
        &self,
        bookmark: Bookmark,
        processed: &mut HashSet<String>,
    ) -> Result<String, String> {
        let mut cmds: Vec<String> = Vec::new();

        if !processed.insert(bookmark.name.clone()) {
            return Err(format!(
                "Circular dependency detected for bookmark '{}'",
                bookmark.name
            ));
        }

        for cmd in bookmark.cmds.iter() {
            if let Some(dep_bookmark_name) = cmd.strip_prefix("bookmark::") {
                if let Some(dep_bookmark) = self
                    .config
                    .bookmarks
                    .iter()
                    .find(|b| b.name == dep_bookmark_name)
                {
                    let mut dep_bookmark = dep_bookmark.clone();
                    dep_bookmark.vars.extend(bookmark.vars.clone());
                    let cmds_from_dep_bookmark =
                        self.gen_template_command(dep_bookmark, processed)?;
                    cmds.push(cmds_from_dep_bookmark);
                } else {
                    return Err(format!("Bookmark '{}' not found", dep_bookmark_name));
                }
            } else if let Some(cmd_key) = cmd.strip_prefix("cmd::") {
                if let Some(cmd_value) = self.config.cmds.get(cmd_key) {
                    let rendered_cmd = self.gen_template_with_vars(cmd_value, &bookmark)?;
                    cmds.push(rendered_cmd);
                } else {
                    return Err(format!("Command key '{}' not found in cmds", cmd_key));
                }
            } else {
                let rendered_cmd = self.gen_template_with_vars(cmd, &bookmark)?;
                cmds.push(rendered_cmd);
            }
        }

        Ok(cmds.join(" \\\n&& "))
    }

    fn gen_template_with_vars(
        &self,
        template: &str,
        bookmark: &Bookmark,
    ) -> Result<String, String> {
        let handlebars = Handlebars::new();
        let mut vars = self.config.vars.clone();
        vars.extend(bookmark.vars.clone());

        handlebars
            .render_template(template, &vars)
            .map(|s| s.trim_start().trim_end().to_string())
            .map_err(|e| format!("Template rendering error: {}", e))
    }

    fn gen_command(&self, bookmark: &Bookmark) -> Result<String, String> {
        let mut processed = HashSet::new();
        let mut cmd = self.gen_template_command(bookmark.clone(), &mut processed)?;

        let exec = bookmark.exec.unwrap_or(self.exec);

        if exec {
            cmd.push('\n');
        }

        Ok(cmd)
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::WriteToStdin,
            PermissionType::OpenFiles,
        ]);

        if let Some(value) = configuration.get(CONFIGURATION_EXEC) {
            self.exec = value.trim().parse::<bool>().unwrap_or_else(|_| {
                self.handle_error(
                    format!("'{CONFIGURATION_EXEC}' config value must be 'true' or 'false', but it's '{value}'. The false is used.")
                );
                false
            })
        }

        if let Some(value) = configuration.get(CONFIGURATION_IGNORE_CASE) {
            self.ignore_case = value.trim().parse::<bool>().unwrap_or_else(|_| {
                self.handle_error(
                    format!("'{CONFIGURATION_IGNORE_CASE}' config value must be 'true' or 'false', but it's '{value}'. The true is used.")
                );
                true
            })
        }

        if let Some(value) = configuration.get(CONFIGURATION_AUTODETECT_FILTER_MODE) {
            self.detect_filter_mode = value.trim().parse::<bool>().unwrap_or_else(|_| {
                self.handle_error(
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
            self.handle_crit_error(format!("Failed to create file '{}': {}.", self.filename, e));
        }

        if let Err(e) = self.load_config() {
            self.handle_error(format!(
                "Failed to load config file '{}': {}.",
                self.filename, e
            ));
        }

        match Keybindings::new(configuration) {
            Ok(kb) => self.keybindings = kb,
            Err(e) => {
                self.handle_error(format!(
                    "Failed to parse zellij-bookmarks keybindings, check your config: {}. Default is used.", e
                ));
            }
        }

        subscribe(&[EventType::Key]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;

        if let Event::Key(key) = event {
            match key.bare_key {
                // Not configurable keys
                BareKey::Esc => close_focus(),
                BareKey::Char('c') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                    close_focus();
                }
                BareKey::Down | BareKey::Tab => match self.mode {
                    Mode::Bookmarks => {
                        self.bookmarks_mgr.select_down();
                        should_render = true;
                    }
                    Mode::Labels => {
                        self.labels_mgr.select_down();
                        should_render = true;
                    }
                    _ => {}
                },
                BareKey::Up => match self.mode {
                    Mode::Bookmarks => {
                        self.bookmarks_mgr.select_up();
                        should_render = true;
                    }
                    Mode::Labels => {
                        self.labels_mgr.select_up();
                        should_render = true;
                    }
                    _ => {}
                },
                BareKey::Right => {
                    self.mode = self.mode.next();
                    self.filter_mode = filters::Mode::default();
                    self.set_filter();
                    should_render = true;
                }
                BareKey::Left => {
                    self.mode = self.mode.prev();
                    self.filter_mode = filters::Mode::default();
                    self.set_filter();
                    should_render = true;
                }
                BareKey::Char(c)
                    if key.has_modifiers(&[KeyModifier::Ctrl]) && c.is_ascii_digit() =>
                {
                    if let Some(digit) = c.to_digit(10) {
                        if let Ok(mode) = Mode::try_from(digit) {
                            if self.mode != mode {
                                self.mode = mode;
                                self.filter_mode = filters::Mode::default();
                                self.set_filter();
                                should_render = true;
                            }
                        }
                    }
                }
                BareKey::Char(c) if key.has_no_modifiers() => match self.mode {
                    Mode::Bookmarks | Mode::Labels => {
                        if self.detect_filter_mode && self.filter.is_empty() {
                            if c.is_ascii_digit() {
                                self.filter_mode = filters::Mode::ID
                            } else if self.filter_mode == filters::Mode::ID {
                                self.filter_mode = filters::Mode::Name
                            }
                        }
                        match self.filter_mode {
                            filters::Mode::ID => {
                                if let Some(digit) = c.to_digit(10) {
                                    if !self.filter.is_empty() || digit > 0 {
                                        self.filter.push(c);

                                        self.set_filter();

                                        should_render = true;
                                    }
                                }
                            }
                            _ => {
                                self.filter.push(c);

                                self.set_filter();

                                should_render = true;
                            }
                        }
                    }
                    _ => {}
                },
                BareKey::Backspace => match self.mode {
                    Mode::Bookmarks | Mode::Labels => {
                        self.filter.pop();

                        self.set_filter();

                        should_render = true;
                    }
                    _ => {}
                },
                BareKey::Enter => match self.mode {
                    Mode::Bookmarks => {
                        match self.bookmarks_mgr.get_selected() {
                            Some(bookmark) => match self.gen_command(bookmark) {
                                Ok(cmd) => {
                                    close_focus();
                                    write_chars(cmd.as_str());
                                }
                                Err(err) => {
                                    self.handle_error(format!(
                                        "Failed to generate command: {}",
                                        err
                                    ));
                                    should_render = true;
                                }
                            },
                            None => should_render = true,
                        };
                    }
                    Mode::Labels => {
                        self.filter_mode = filters::Mode::Label;
                        self.filter = match self.labels_mgr.get_selected() {
                            Some(label) => label.name.clone(),
                            None => String::new(),
                        };
                        self.mode = Mode::Bookmarks;
                        self.view_desc = false;

                        self.set_filter();

                        should_render = true;
                    }
                    _ => {}
                },
                _ => {
                    // Configurable keys
                    if self.keybindings.edit.matches(&key) {
                        let file = FileToOpen::new(self.filename.as_str()).with_cwd(self.get_cwd());
                        open_file_in_place(file, Default::default());
                    } else if self.keybindings.reload.matches(&key) {
                        if let Err(e) = self.load_config() {
                            self.handle_error(format!(
                                "Failed to load config file '{}': {}.",
                                self.get_path().display(),
                                e
                            ));
                        }

                        self.filter = "".to_string();

                        self.reset_selection();

                        should_render = true;
                    } else if self.keybindings.switch_filter_label.matches(&key) {
                        if self.mode == Mode::Bookmarks {
                            self.filter_mode = self.filter_mode.switch_to(filters::Mode::Label);
                            self.set_filter();
                            should_render = true;
                        }
                    } else if self.keybindings.switch_filter_id.matches(&key) {
                        match self.mode {
                            Mode::Bookmarks | Mode::Labels => {
                                self.filter_mode = self.filter_mode.switch_to(filters::Mode::ID);
                                self.set_filter();
                                should_render = true;
                            }
                            _ => {}
                        }
                    } else if self.keybindings.describe.matches(&key) {
                        #[allow(clippy::collapsible_if)]
                        if self.mode == Mode::Bookmarks {
                            self.view_desc = !self.view_desc;
                            should_render = true;
                        }
                    }
                }
            }
        };

        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        if rows < RESERVE_ROW_COUNT || cols < RESERVE_COLUMN_COUNT {
            eprintln!(
                "\
                ERROR: Rendering failed. The panel is too small.\
                The number of rows must be more than {RESERVE_ROW_COUNT}, but it's '{rows}.\
                The number of columns must be more than {RESERVE_COLUMN_COUNT}, but it's '{cols}"
            );
            close_focus()
        }
        if self.render_errors() {
            return;
        }
        match self.mode {
            Mode::Bookmarks => {
                self.render_bookmarks(rows, cols);
            }
            Mode::Labels => {
                self.render_labels(rows, cols);
            }
            Mode::Usage => {
                self.render_usage();
            }
        }
    }
}

register_plugin!(State);
