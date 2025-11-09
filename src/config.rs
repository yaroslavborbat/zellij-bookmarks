use super::bookmark::Bookmark;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type BookmarkList = Vec<Bookmark>;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Config {
    #[serde(default)]
    pub vars: HashMap<String, String>,
    #[serde(default)]
    pub cmds: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_bookmarks")]
    pub bookmarks: BookmarkList,
}

fn deserialize_bookmarks<'de, D>(
    deserializer: D,
) -> zellij_tile::prelude::Result<BookmarkList, D::Error>
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
