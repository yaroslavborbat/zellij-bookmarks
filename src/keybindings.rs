use crate::core::keybinding_parser::{parse_key_info, Keybinding};
use std::collections::BTreeMap;
use zellij_tile::prelude::*;

const BIND_EDIT: &str = "bind_edit";
const BIND_RELOAD: &str = "bind_reload";
const BIND_SWITCH_FILTER_LABEL: &str = "bind_switch_filter_label";
const BIND_SWITCH_FILTER_ID: &str = "bind_switch_filter_id";
const BIND_DESCRIBE: &str = "bind_describe";

#[derive(Clone)]
pub(crate) struct Keybindings {
    pub edit: Keybinding,
    pub reload: Keybinding,
    pub switch_filter_label: Keybinding,
    pub switch_filter_id: Keybinding,
    pub describe: Keybinding,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            edit: Keybinding::new(KeyModifier::Ctrl, 'e'),
            reload: Keybinding::new(KeyModifier::Ctrl, 'r'),
            switch_filter_label: Keybinding::new(KeyModifier::Ctrl, 'l'),
            switch_filter_id: Keybinding::new(KeyModifier::Ctrl, 'i'),
            describe: Keybinding::new(KeyModifier::Ctrl, 'd'),
        }
    }
}

impl Keybindings {
    pub fn new(conf: BTreeMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut default = Self::default();

        if let Some(value) = conf.get(BIND_EDIT) {
            default.edit = parse_key_info(value)?
        }
        if let Some(value) = conf.get(BIND_RELOAD) {
            default.reload = parse_key_info(value)?
        }
        if let Some(value) = conf.get(BIND_SWITCH_FILTER_LABEL) {
            default.switch_filter_label = parse_key_info(value)?
        }
        if let Some(value) = conf.get(BIND_SWITCH_FILTER_ID) {
            default.switch_filter_id = parse_key_info(value)?
        }
        if let Some(value) = conf.get(BIND_DESCRIBE) {
            default.describe = parse_key_info(value)?
        }
        Ok(default)
    }
}
