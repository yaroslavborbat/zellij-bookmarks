mod filters;
mod tab_manager;

use crate::filters::{BookmarkFilter, Filter, LabelFilter};
use crate::tab_manager::TabManager;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fmt::Debug;
use std::io;
use std::io::{Read, Write};
use std::{fs, path};
use zellij_tile::prelude::*;

const CONFIGURATION_EXEC: &str = "exec";
const CONFIGURATION_IGNORE_CASE: &str = "ignore_case";
const CONFIGURATION_FILENAME: &str = "filename";

const CWD: &str = "/host";

const BASE_COLOR: usize = 2;

const RESERVE_ROW_COUNT: usize = 5;
const RESERVE_COLUMN_COUNT: usize = 12;

type Bookmarks = Vec<Bookmark>;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct Bookmark {
    name: String,
    command: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    exec: Option<bool>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct Config {
    bookmarks: Bookmarks,
}

#[derive(Default)]
struct State {
    exec: bool,
    ignore_case: bool,
    filename: String,
    filter: String,
    bookmarks_mgr: TabManager<Bookmark>,
    labels_mgr: TabManager<String>,
    filter_by_label: bool,
    view_command: bool,
    view_labels: bool,
    view_usage: bool,
    crit_error_message: Option<String>,
    error_message: Option<String>,
}

impl State {
    fn bookmark_filter(&self) -> Box<dyn Filter<Bookmark>> {
        Box::new(BookmarkFilter::new(
            self.filter.clone(),
            self.filter_by_label,
            self.ignore_case,
        ))
    }

    fn label_filter(&self) -> Box<dyn Filter<String>> {
        Box::new(LabelFilter::new(self.filter.clone(), self.ignore_case))
    }

    fn set_filter(&mut self) {
        if self.view_labels {
            self.labels_mgr.with_filter(self.label_filter());
        }
        self.bookmarks_mgr.with_filter(self.bookmark_filter());
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
            let serialized = serde_yaml::to_string(&conf).expect("Failed to serialize bookmarks");
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
        self.bookmarks_mgr = TabManager::new(config.bookmarks);

        let mut set = HashSet::new();

        for (_, bookmark) in self.bookmarks_mgr.iter() {
            if !bookmark.label.is_empty() {
                set.insert(bookmark.label.clone());
            }
        }
        self.labels_mgr = TabManager::new(set.into_iter().collect());

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

    fn render_usage(&self) -> bool {
        if !self.view_usage {
            return false;
        }
        render_mode(2, 0, "Usage".to_string());

        let mut table = Table::new();
        table = table.add_row(vec!["KeyBindings", "Action"]);
        table = table.add_row(vec![
            "Ctrl + e",
            "Open the bookmark configuration file for editing.",
        ]);
        table = table.add_row(vec![
            "Ctrl + r",
            "Reload bookmarks. Required after making changes to the bookmarks configuration.",
        ]);
        table = table.add_row(vec![
            "Ctrl + f",
            "Toggle filter modes (available only in bookmark mode).",
        ]);
        table = table.add_row(vec![
            "Ctrl + l",
            "Toggle between Bookmarks and Labels modes.",
        ]);
        table = table.add_row(vec![
            "Ctrl + v",
            "Display the command associated with the bookmark (only for bookmarks).",
        ]);
        table = table.add_row(vec!["Ctrl + u", "Display usage instructions."]);

        print_table_with_coordinates(table, 2, 2, None, None);
        true
    }

    fn render_labels(&self, rows: usize, cols: usize) -> bool {
        if !self.view_labels {
            return false;
        }
        self.render_main_menu(
            rows,
            cols,
            self.labels_mgr.get_position(),
            self.labels_mgr.len(),
            "Labels".to_string(),
            false,
            self.labels_mgr.iter(),
        );
        true
    }

    fn render_bookmarks(&self, rows: usize, cols: usize) -> bool {
        let iter = self.bookmarks_mgr.iter().map(|(i, b)| {
            if self.view_command {
                (i, &b.command)
            } else {
                (i, &b.name)
            }
        });
        self.render_main_menu(
            rows,
            cols,
            self.bookmarks_mgr.get_position(),
            self.bookmarks_mgr.len(),
            "Bookmarks".to_string(),
            self.filter_by_label,
            iter,
        );
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn render_main_menu<'a>(
        &self,
        rows: usize,
        cols: usize,
        selected: usize,
        length: usize,
        mode: String,
        filter_by_label: bool,
        iterator: impl Iterator<Item = (usize, &'a String)>,
    ) {
        let (x, y, width, height) = self.main_menu_size(rows, cols);

        render_mode(x + 2, y, mode);

        render_search_block(x + 2, y + 1, self.filter.clone(), filter_by_label);

        let (begin, end) = if selected >= height {
            (selected + 1 - height, selected)
        } else {
            (0, height - 1)
        };

        render_right_counter(begin, width, y + 2);

        {
            let mut number = y + 3;

            for (i, value) in iterator {
                if i < begin {
                    continue;
                }
                if i > end {
                    break;
                }
                let text = prepare_row_text(value.clone(), i + 1, width, selected == i);

                print_text_with_coordinates(text, x, number, None, None);

                number += 1;
            }
        }

        render_all_counter(x + 2, rows, length);

        if length > end {
            render_right_counter_with_max(length - 1 - end, length, width, rows);
        }
    }

    fn main_menu_size(&self, rows: usize, cols: usize) -> (usize, usize, usize, usize) {
        // x, y, width, height
        let width = cols;
        let x = 0;
        let y = 0;
        let height = rows.saturating_sub(RESERVE_ROW_COUNT);

        (x, y, width, height)
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

        self.ignore_case = true;
        if let Some(value) = configuration.get(CONFIGURATION_IGNORE_CASE) {
            self.ignore_case = value.trim().parse::<bool>().unwrap_or_else(|_| {
                self.handle_error(
                    format!("'{CONFIGURATION_IGNORE_CASE}' config value must be 'true' or 'false', but it's '{value}'. The true is used.")
                );
                true
            })
        }

        self.filename = ".zellij_bookmarks.yaml".to_string();
        if let Some(value) = configuration.get(CONFIGURATION_FILENAME) {
            if !value.is_empty() {
                self.filename = value.clone();
            }
        }

        if let Err(e) = self.create_config_if_not_exists() {
            self.handle_crit_error(format!("failed to create file '{}': {}", self.filename, e));
        }

        if let Err(e) = self.load_config() {
            self.handle_error(format!(
                "failed to load config file '{}': {}",
                self.filename, e
            ));
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
                BareKey::Down | BareKey::Tab => {
                    if self.view_labels {
                        self.labels_mgr.select_down();
                    } else {
                        self.bookmarks_mgr.select_down();
                    }

                    should_render = true;
                }
                BareKey::Up => {
                    if self.view_labels {
                        self.labels_mgr.select_up();
                    } else {
                        self.bookmarks_mgr.select_up();
                    }

                    should_render = true;
                }
                BareKey::Char(c)
                    if !key.has_modifiers(&[KeyModifier::Ctrl])
                        && (c.is_ascii_alphabetic() || c.is_ascii_digit()) =>
                {
                    self.filter.push(c);

                    self.set_filter();

                    should_render = true;
                }
                BareKey::Backspace => {
                    self.filter.pop();

                    self.set_filter();

                    should_render = true;
                }
                // Enter
                BareKey::Char('\n') => {
                    if self.view_labels {
                        self.filter_by_label = true;
                        self.filter = match self.labels_mgr.get_selected() {
                            Some(label) => label.clone(),
                            None => String::new(),
                        };
                        self.view_labels = false;
                        self.view_command = false;

                        self.set_filter();

                        should_render = true;
                    } else {
                        let bookmark = match self.bookmarks_mgr.get_selected() {
                            Some(bookmark) => bookmark,
                            None => return true,
                        };
                        let mut cmd = bookmark
                            .command
                            .trim_start_matches('\n')
                            .trim_end_matches('\n')
                            .to_string();

                        let exec = match bookmark.exec {
                            Some(value) => value,
                            None => self.exec,
                        };

                        if exec {
                            cmd.push('\n');
                        }
                        close_focus();

                        write_chars(cmd.as_str());
                    }
                }
                // Configurable keys
                BareKey::Char('e') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                    let file = FileToOpen::new(self.filename.as_str()).with_cwd(self.get_cwd());
                    open_file_in_place(file, Default::default());
                }
                BareKey::Char('r') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                    if let Err(e) = self.load_config() {
                        self.handle_error(format!(
                            "failed to load config file '{}': {}",
                            self.get_path().display(),
                            e
                        ));
                    }

                    self.filter = "".to_string();

                    self.reset_selection();

                    should_render = true;
                }
                BareKey::Char('f') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                    self.filter_by_label = !self.filter_by_label;

                    should_render = true;
                }
                BareKey::Char('l') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                    self.view_labels = !self.view_labels;

                    self.set_filter();

                    should_render = true;
                }
                BareKey::Char('v') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                    self.view_command = !self.view_command;

                    should_render = true;
                }
                BareKey::Char('u') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                    self.view_usage = !self.view_usage;

                    should_render = true;
                }
                _ => (),
            }
        };

        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        if rows < RESERVE_ROW_COUNT || cols < RESERVE_COLUMN_COUNT {
            eprintln!(
                "\
                ERROR: Rendering failed. The panel is too small. \
                The number of rows must be more than {RESERVE_ROW_COUNT}, but it's '{rows}. \
                The number of columns must be more than {RESERVE_COLUMN_COUNT}, but it's '{cols}"
            );
            close_focus()
        }
        if self.render_errors() {
            return;
        }
        if self.render_usage() {
            return;
        }
        if self.render_labels(rows, cols) {
            return;
        }
        self.render_bookmarks(rows, cols);
    }
}

register_plugin!(State);

fn prepare_row_text(row: String, position: usize, max_length: usize, selected: bool) -> Text {
    let truncated_row = {
        let formatted = format!("{}. {}", position, row);
        if formatted.len() > max_length {
            let truncated_len = max_length.saturating_sub(3);
            let mut truncated_str = formatted.chars().take(truncated_len).collect::<String>();
            truncated_str.push_str("...");
            truncated_str
        } else {
            formatted
        }
    };
    let text = if selected {
        Text::new(truncated_row.yellow().to_string()).selected()
    } else {
        Text::new(truncated_row)
    };
    text
}

fn render_mode(x: usize, y: usize, mode: String) {
    let s = format!("Mode: {}", mode);
    let text = Text::new(s).color_range(BASE_COLOR, ..4);
    print_text_with_coordinates(text, x, y, None, None)
}

fn render_search_block(x: usize, y: usize, filter: String, filter_by_label: bool) {
    let search = if filter_by_label {
        "Search (by label):".to_string()
    } else {
        "Search (by name):".to_string()
    };
    let filter = format!("{} {}_", search, filter.clone());

    let text = Text::new(filter).color_range(BASE_COLOR, ..6);
    print_text_with_coordinates(text, x, y, None, None);
}

// Render row with All row-counter
fn render_all_counter(x: usize, y: usize, all: usize) {
    let all_count = format!("All: {}", all);
    let text = Text::new(all_count).color_range(BASE_COLOR, ..);
    print_text_with_coordinates(text, x, y, None, None);
}

// Render row with right counter with max
fn render_right_counter_with_max(count: usize, max_count: usize, width: usize, y: usize) {
    if count == max_count {
        return;
    }
    render_right_counter(count, width, y);
}

// Render row with right counter
fn render_right_counter(count: usize, width: usize, y: usize) {
    if count == 0 {
        return;
    }
    let row = format!("+ {} more  ", count);
    let x = width - row.len();
    let text = Text::new(row.yellow().bold().to_string());
    print_text_with_coordinates(text, x, y, None, None);
}
