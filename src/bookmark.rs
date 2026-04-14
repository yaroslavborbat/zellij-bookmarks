use crate::core::{IdGetter, LabelsGetter, NameGetter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Bookmark {
    #[serde(default)]
    pub id: usize,
    pub name: String,
    #[serde(default)]
    pub desc: String,
    pub cmds: Vec<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub vars: HashMap<String, String>,
    pub exec: Option<bool>,
    pub separator: Option<String>,
}

impl IdGetter for Bookmark {
    fn get_id(&self) -> usize {
        self.id
    }
}

impl NameGetter for Bookmark {
    fn get_name(&self) -> String {
        self.name.to_string()
    }
}

impl LabelsGetter for Bookmark {
    fn get_labels(&self) -> Vec<String> {
        self.labels.clone()
    }
}
