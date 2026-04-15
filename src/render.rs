use crate::core::keybinding_parser::Keybinding;
use crate::core::{render_main_menu, render_mode, RESERVE_COLUMN_COUNT, RESERVE_ROW_COUNT};
use zellij_tile::prelude::*;

use super::{Mode, Navigation, State};

impl State {
    fn render_usage(&self) {
        let all_modes: Vec<Mode> = Mode::iter().collect();
        render_mode(0, 0, Mode::Usage, &all_modes, &self.ui_style);

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
            BareKey::Enter.to_string().as_str(),
            "Open the selected config file in an editor.",
            Mode::Edit.to_string().as_str(),
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
        table = table.add_row(vec![
            format!("{:?} {}", KeyModifier::Ctrl, Mode::Edit as u32).as_str(),
            "Switch to Edit mode to choose a config file.",
            "*",
            "False",
        ]);

        // Configurable
        table = table.add_row(vec![
            self.keybindings.edit.to_string().as_str(),
            "Switch to Edit mode to choose a config file.",
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
        let iter = self
            .labels
            .iter()
            .map(|(index, item)| (index, item.value.id, &item.value.name, item.indices.clone()));
        let all_modes: Vec<Mode> = Mode::iter().collect();

        render_main_menu(
            rows,
            cols,
            self.labels.get_position(),
            self.labels.len(),
            Mode::Labels,
            &all_modes,
            &self.ui_style,
            self.filter.clone(),
            self.filter_mode.to_string(),
            iter,
        );
    }

    fn render_edit(&self, rows: usize, cols: usize) {
        let iter = self
            .editable_files
            .iter()
            .map(|(index, item)| (index, item.value.id, &item.value.path, item.indices.clone()));
        let all_modes: Vec<Mode> = Mode::iter().collect();

        render_main_menu(
            rows,
            cols,
            self.editable_files.get_position(),
            self.editable_files.len(),
            Mode::Edit,
            &all_modes,
            &self.ui_style,
            self.filter.clone(),
            self.filter_mode.to_string(),
            iter,
        );
    }

    fn render_bookmarks(&self, rows: usize, cols: usize) {
        let iter = self.bookmarks.iter().map(|(index, item)| {
            if self.view_desc {
                (index, item.value.id, &item.value.desc, item.indices.clone())
            } else {
                (index, item.value.id, &item.value.name, item.indices.clone())
            }
        });
        let all_modes: Vec<Mode> = Mode::iter().collect();
        render_main_menu(
            rows,
            cols,
            self.bookmarks.get_position(),
            self.bookmarks.len(),
            Mode::Bookmarks,
            &all_modes,
            &self.ui_style,
            self.filter.clone(),
            self.filter_mode.to_string(),
            iter,
        );
    }

    pub(crate) fn render(&mut self, rows: usize, cols: usize) {
        if rows < RESERVE_ROW_COUNT || cols < RESERVE_COLUMN_COUNT {
            eprintln!(
                "\
                ERROR: Rendering failed. The panel is too small.\
                The number of rows must be more than {}, but it's '{}.\
                The number of columns must be more than {}, but it's '{}",
                RESERVE_ROW_COUNT, rows, RESERVE_COLUMN_COUNT, cols
            );
            close_focus()
        }
        if self.error_mgr.render() {
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
            Mode::Edit => {
                self.render_edit(rows, cols);
            }
        }
    }
}
