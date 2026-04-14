mod bookmark;
mod config;
mod core;
mod keybindings;
mod label;
mod load;
mod render;
mod update;

use crate::bookmark::Bookmark;
use crate::config::Config;
use crate::core::{ErrorManager, FilterMode, FilteredList};
use crate::keybindings::Keybindings;
use crate::label::Label;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::{fmt, path};
use zellij_tile::prelude::*;

pub const CWD: &str = "/host";

struct State {
    mode: Mode,
    exec: bool,
    separator: String,
    ignore_case: bool,
    detect_filter_mode: bool,
    fuzzy_search: bool,
    view_desc: bool,
    filter_mode: FilterMode,
    filter: String,
    filename: String,
    config: Config,
    keybindings: Keybindings,
    bookmarks: FilteredList<Bookmark>,
    labels: FilteredList<Label>,
    error_mgr: ErrorManager,
}

impl Default for State {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            exec: false,
            separator: " \\\n&& ".to_string(),
            ignore_case: true,
            detect_filter_mode: true,
            fuzzy_search: true,
            view_desc: false,
            filter_mode: Default::default(),
            filter: "".to_string(),
            filename: ".zellij_bookmarks.yaml".to_string(),
            config: Default::default(),
            keybindings: Default::default(),
            bookmarks: Default::default(),
            labels: Default::default(),
            error_mgr: ErrorManager::new(),
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
    fn get_cwd(&self) -> path::PathBuf {
        path::PathBuf::from(CWD)
    }

    fn get_path(&self) -> path::PathBuf {
        self.get_cwd().join(self.filename.as_str())
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.load(configuration);
    }

    fn update(&mut self, event: Event) -> bool {
        self.update(event)
    }

    fn render(&mut self, rows: usize, cols: usize) {
        self.render(rows, cols);
    }
}

register_plugin!(State);
