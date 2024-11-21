use crate::Bookmark;

pub(crate) trait Filter<T> {
    fn keep(&self, t: &T) -> bool;
}

pub(crate) struct LabelFilter {
    filter: String,
    ignore_case: bool,
}

impl LabelFilter {
    pub(crate) fn new(filter: String, ignore_case: bool) -> Self {
        LabelFilter {
            filter,
            ignore_case,
        }
    }
}

impl Filter<String> for LabelFilter {
    fn keep(&self, label: &String) -> bool {
        let (filter, l) = if self.ignore_case {
            (self.filter.to_lowercase(), label.to_lowercase())
        } else {
            (self.filter.clone(), label.clone())
        };
        l == filter || l.contains(&filter)
    }
}

pub(crate) struct BookmarkFilter {
    filter: String,
    by_label: bool,
    ignore_case: bool,
}

impl BookmarkFilter {
    pub(crate) fn new(filter: String, by_label: bool, ignore_case: bool) -> Self {
        BookmarkFilter {
            filter,
            by_label,
            ignore_case,
        }
    }
}

impl Filter<Bookmark> for BookmarkFilter {
    fn keep(&self, bookmark: &Bookmark) -> bool {
        let filter = if self.ignore_case {
            self.filter.to_lowercase()
        } else {
            self.filter.clone()
        };

        let target = if self.by_label {
            let label = &bookmark.label;
            if self.ignore_case {
                label.to_lowercase()
            } else {
                label.clone()
            }
        } else {
            let name = &bookmark.name;
            if self.ignore_case {
                name.to_lowercase()
            } else {
                name.clone()
            }
        };

        target == filter || target.contains(&filter)
    }
}
