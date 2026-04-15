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

impl Config {
    pub(crate) fn merge(&mut self, other: Config) -> Result<(), String> {
        for key in other.vars.keys() {
            if self.vars.contains_key(key) {
                return Err(format!("Duplicate var name: {}", key));
            }
        }

        for key in other.cmds.keys() {
            if self.cmds.contains_key(key) {
                return Err(format!("Duplicate cmd name: {}", key));
            }
        }

        let mut bookmark_names: HashSet<String> = self
            .bookmarks
            .iter()
            .map(|bookmark| bookmark.name.clone())
            .collect();
        for bookmark in &other.bookmarks {
            if !bookmark_names.insert(bookmark.name.clone()) {
                return Err(format!("Duplicate bookmark name: {}", bookmark.name));
            }
        }

        self.vars.extend(other.vars);
        self.cmds.extend(other.cmds);
        self.bookmarks.extend(other.bookmarks);
        self.reindex_bookmarks();

        Ok(())
    }

    fn reindex_bookmarks(&mut self) {
        for (i, bookmark) in self.bookmarks.iter_mut().enumerate() {
            bookmark.id = i + 1;
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{BookmarkList, Config};
    use crate::bookmark::Bookmark;
    use std::collections::HashMap;

    fn bookmark(name: &str) -> Bookmark {
        Bookmark {
            name: name.to_string(),
            cmds: vec!["echo test".to_string()],
            ..Default::default()
        }
    }

    #[test]
    fn merge_appends_unique_entries_and_reindexes_bookmarks() {
        let mut base = Config {
            vars: HashMap::from([(String::from("base"), String::from("value"))]),
            cmds: HashMap::from([(String::from("hello"), String::from("echo base"))]),
            bookmarks: BookmarkList::from([bookmark("base")]),
        };
        let extra = Config {
            vars: HashMap::from([(String::from("extra"), String::from("value"))]),
            cmds: HashMap::from([(String::from("world"), String::from("echo extra"))]),
            bookmarks: BookmarkList::from([bookmark("extra")]),
        };

        base.merge(extra).unwrap();

        assert_eq!(base.vars.get("base"), Some(&String::from("value")));
        assert_eq!(base.vars.get("extra"), Some(&String::from("value")));
        assert_eq!(base.cmds.get("hello"), Some(&String::from("echo base")));
        assert_eq!(base.cmds.get("world"), Some(&String::from("echo extra")));
        assert_eq!(base.bookmarks.len(), 2);
        assert_eq!(base.bookmarks[0].id, 1);
        assert_eq!(base.bookmarks[1].id, 2);
    }

    #[test]
    fn merge_rejects_duplicate_bookmark_names() {
        let mut base = Config {
            vars: HashMap::new(),
            cmds: HashMap::new(),
            bookmarks: BookmarkList::from([bookmark("dup")]),
        };
        let extra = Config {
            vars: HashMap::new(),
            cmds: HashMap::new(),
            bookmarks: BookmarkList::from([bookmark("dup")]),
        };

        let err = base.merge(extra).unwrap_err();

        assert!(err.contains("Duplicate bookmark name: dup"));
    }

    #[test]
    fn merge_rejects_duplicate_var_names() {
        let mut base = Config {
            vars: HashMap::from([(String::from("shared"), String::from("base"))]),
            cmds: HashMap::new(),
            bookmarks: BookmarkList::new(),
        };
        let extra = Config {
            vars: HashMap::from([(String::from("shared"), String::from("extra"))]),
            cmds: HashMap::new(),
            bookmarks: BookmarkList::new(),
        };

        let err = base.merge(extra).unwrap_err();

        assert!(err.contains("Duplicate var name: shared"));
    }

    #[test]
    fn merge_rejects_duplicate_cmd_names() {
        let mut base = Config {
            vars: HashMap::new(),
            cmds: HashMap::from([(String::from("hello"), String::from("echo base"))]),
            bookmarks: BookmarkList::new(),
        };
        let extra = Config {
            vars: HashMap::new(),
            cmds: HashMap::from([(String::from("hello"), String::from("echo extra"))]),
            bookmarks: BookmarkList::new(),
        };

        let err = base.merge(extra).unwrap_err();

        assert!(err.contains("Duplicate cmd name: hello"));
    }
}
