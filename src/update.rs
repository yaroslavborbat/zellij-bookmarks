use super::{bookmark, Mode, Navigation, State};
use crate::bookmark::Bookmark;
use crate::core::{Filter, FilterMode, GenericFilter};
use crate::label::Label;
use handlebars::Handlebars;
use std::collections::HashSet;
use zellij_tile::prelude::*;

impl State {
    fn bookmark_filter(&self) -> Box<dyn Filter<Bookmark>> {
        Box::new(GenericFilter::new(
            self.filter_mode,
            self.filter.clone(),
            self.ignore_case,
            self.fuzzy_search,
        ))
    }

    fn label_filter(&self) -> Box<dyn Filter<Label>> {
        Box::new(GenericFilter::new(
            self.filter_mode,
            self.filter.clone(),
            self.ignore_case,
            self.fuzzy_search,
        ))
    }

    fn set_filter(&mut self) {
        match self.mode {
            Mode::Bookmarks => self.bookmarks.with_filter(self.bookmark_filter()),
            Mode::Labels => self.labels.with_filter(self.label_filter()),
            _ => {}
        }
    }

    fn reset_selection(&mut self) {
        self.bookmarks.reset_selection();
        self.labels.reset_selection();
    }

    fn gen_template_command(
        &self,
        bookmark: bookmark::Bookmark,
        processed: &mut HashSet<String>,
    ) -> Result<String, String> {
        let mut cmds: Vec<String> = Vec::new();
        let separator = bookmark
            .separator
            .clone()
            .unwrap_or_else(|| self.separator.clone());

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

        Ok(cmds.join(separator.as_str()))
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

    fn gen_command(&self, bookmark: &bookmark::Bookmark) -> Result<String, String> {
        let mut processed = HashSet::new();
        let mut cmd = self.gen_template_command(bookmark.clone(), &mut processed)?;

        let exec = bookmark.exec.unwrap_or(self.exec);

        if exec {
            cmd.push('\n');
        }

        Ok(cmd)
    }

    pub(crate) fn update(&mut self, event: Event) -> bool {
        if let Event::Key(key) = event {
            self.handle_key_event(key)
        } else {
            false
        }
    }

    fn handle_key_event(&mut self, key: KeyWithModifier) -> bool {
        let mut should_render = false;

        match key.bare_key {
            // Not configurable keys
            BareKey::Esc => close_focus(),
            BareKey::Char('c') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                close_focus();
            }
            BareKey::Down | BareKey::Tab => match self.mode {
                Mode::Bookmarks => {
                    self.bookmarks.select_down();
                    should_render = true;
                }
                Mode::Labels => {
                    self.labels.select_down();
                    should_render = true;
                }
                _ => {}
            },
            BareKey::Up => match self.mode {
                Mode::Bookmarks => {
                    self.bookmarks.select_up();
                    should_render = true;
                }
                Mode::Labels => {
                    self.labels.select_up();
                    should_render = true;
                }
                _ => {}
            },
            BareKey::Right => {
                self.mode = self.mode.next();
                self.filter_mode = FilterMode::default();
                self.set_filter();
                should_render = true;
            }
            BareKey::Left => {
                self.mode = self.mode.prev();
                self.filter_mode = FilterMode::default();
                self.set_filter();
                should_render = true;
            }
            BareKey::Char(c) if key.has_modifiers(&[KeyModifier::Ctrl]) && c.is_ascii_digit() => {
                if let Some(digit) = c.to_digit(10) {
                    if let Ok(mode) = Mode::try_from(digit) {
                        if self.mode != mode {
                            self.mode = mode;
                            self.filter_mode = FilterMode::default();
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
                            self.filter_mode = FilterMode::ID
                        } else if self.filter_mode == FilterMode::ID {
                            self.filter_mode = FilterMode::Name
                        }
                    }
                    match self.filter_mode {
                        FilterMode::ID => {
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
                    match self.bookmarks.get_selected() {
                        Some(bookmark) => match self.gen_command(bookmark) {
                            Ok(cmd) => {
                                close_focus();
                                write_chars(cmd.as_str());
                            }
                            Err(err) => {
                                self.error_mgr
                                    .handle_error(format!("Failed to generate command: {}", err));
                                should_render = true;
                            }
                        },
                        None => should_render = true,
                    };
                }
                Mode::Labels => {
                    self.filter_mode = FilterMode::Label;
                    self.filter = match self.labels.get_selected() {
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
                        self.error_mgr.handle_error(format!(
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
                        self.filter_mode = self.filter_mode.switch_to(FilterMode::Label);
                        self.set_filter();
                        should_render = true;
                    }
                } else if self.keybindings.switch_filter_id.matches(&key) {
                    match self.mode {
                        Mode::Bookmarks | Mode::Labels => {
                            self.filter_mode = self.filter_mode.switch_to(FilterMode::ID);
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

        should_render
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use crate::bookmark::Bookmark;
    use crate::config::Config;
    use std::collections::HashMap;

    fn bookmark(name: &str, cmds: &[&str]) -> Bookmark {
        Bookmark {
            name: name.to_string(),
            cmds: cmds.iter().map(|cmd| cmd.to_string()).collect(),
            ..Default::default()
        }
    }

    fn state_with_config(config: Config) -> State {
        State {
            config,
            ..Default::default()
        }
    }

    #[test]
    fn gen_command_expands_nested_bookmarks() {
        let dependency = bookmark("dep", &["echo dep-1", "echo dep-2"]);
        let root = bookmark("root", &["echo start", "bookmark::dep", "echo end"]);
        let state = state_with_config(Config {
            bookmarks: vec![dependency, root.clone()],
            ..Default::default()
        });

        let cmd = state.gen_command(&root).unwrap();

        assert_eq!(
            cmd,
            "echo start \\\n&& echo dep-1 \\\n&& echo dep-2 \\\n&& echo end"
        );
    }

    #[test]
    fn gen_command_expands_named_commands() {
        let root = bookmark("root", &["cmd::prepare", "echo ready", "cmd::finish"]);
        let state = state_with_config(Config {
            cmds: HashMap::from([
                ("prepare".to_string(), "echo prepare".to_string()),
                ("finish".to_string(), "echo finish".to_string()),
            ]),
            bookmarks: vec![root.clone()],
            ..Default::default()
        });

        let cmd = state.gen_command(&root).unwrap();

        assert_eq!(cmd, "echo prepare \\\n&& echo ready \\\n&& echo finish");
    }

    #[test]
    fn gen_command_uses_separator_from_bookmark_or_global_config() {
        let mut root = bookmark("root", &["echo one", "echo two"]);
        root.separator = Some(" || ".to_string());

        let state = State {
            separator: " && ".to_string(),
            config: Config {
                bookmarks: vec![root.clone()],
                ..Default::default()
            },
            ..Default::default()
        };

        let cmd = state.gen_command(&root).unwrap();

        assert_eq!(cmd, "echo one || echo two");
    }

    #[test]
    fn gen_command_uses_global_separator_when_bookmark_override_is_missing() {
        let root = bookmark("root", &["echo one", "echo two"]);
        let state = State {
            separator: " ~~ ".to_string(),
            config: Config {
                bookmarks: vec![root.clone()],
                ..Default::default()
            },
            ..Default::default()
        };

        let cmd = state.gen_command(&root).unwrap();

        assert_eq!(cmd, "echo one ~~ echo two");
    }

    #[test]
    fn gen_command_supports_mixed_bookmark_and_cmd_expansion() {
        let dependency = bookmark("dep", &["cmd::prepare", "echo dep"]);
        let root = bookmark("root", &["echo start", "bookmark::dep", "cmd::finish"]);
        let state = state_with_config(Config {
            cmds: HashMap::from([
                ("prepare".to_string(), "echo prepare".to_string()),
                ("finish".to_string(), "echo finish".to_string()),
            ]),
            bookmarks: vec![dependency, root.clone()],
            ..Default::default()
        });

        let cmd = state.gen_command(&root).unwrap();

        assert_eq!(
            cmd,
            "echo start \\\n&& echo prepare \\\n&& echo dep \\\n&& echo finish"
        );
    }

    #[test]
    fn gen_command_appends_newline_when_exec_is_enabled() {
        let root = bookmark("root", &["echo run"]);
        let state = State {
            exec: true,
            config: Config {
                bookmarks: vec![root.clone()],
                ..Default::default()
            },
            ..Default::default()
        };

        let cmd = state.gen_command(&root).unwrap();

        assert_eq!(cmd, "echo run\n");
    }
}
