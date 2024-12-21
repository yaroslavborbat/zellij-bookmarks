use crate::{Bookmark, Label};
use std::fmt;
use std::fmt::Formatter;

pub(crate) trait Filter<T> {
    fn keep(&self, t: &T) -> bool;
}
#[derive(Default, PartialEq, Copy, Clone)]
pub(crate) enum Mode {
    #[default]
    Name,
    ID,
    Label,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Name => "Name",
            Self::ID => "ID",
            Self::Label => "Label",
        };
        write!(f, "{}", name)
    }
}

impl Mode {
    pub(crate) fn switch_to(&self, mode: Mode) -> Self {
        if *self == mode {
            Mode::default()
        } else {
            mode
        }
    }
}

pub(crate) struct LabelFilter {
    mode: Mode,
    filter: String,
    ignore_case: bool,
}

impl LabelFilter {
    pub(crate) fn new(mode: Mode, filter: String, ignore_case: bool) -> Self {
        LabelFilter {
            mode,
            filter,
            ignore_case,
        }
    }
    fn keep_by_name(&self, label: &Label) -> bool {
        if self.filter.is_empty() {
            return true;
        }
        let (filter, l) = if self.ignore_case {
            (self.filter.to_lowercase(), label.name.to_lowercase())
        } else {
            (self.filter.clone(), label.name.to_owned())
        };
        l == filter || l.contains(&filter)
    }

    fn keep_by_id(&self, label: &Label) -> bool {
        label.id.to_string().starts_with(&self.filter.to_string())
    }
}

impl Filter<Label> for LabelFilter {
    fn keep(&self, label: &Label) -> bool {
        match self.mode {
            Mode::ID => self.keep_by_id(label),
            _ => self.keep_by_name(label),
        }
    }
}

pub(crate) struct BookmarkFilter {
    mode: Mode,
    filter: String,
    ignore_case: bool,
}

impl BookmarkFilter {
    pub(crate) fn new(mode: Mode, filter: String, ignore_case: bool) -> Self {
        BookmarkFilter {
            mode,
            filter,
            ignore_case,
        }
    }

    fn keep_by_label(&self, bookmark: &Bookmark) -> bool {
        let filter = self.get_filter();
        if filter.is_empty() {
            return true;
        }
        for label in bookmark.labels.iter() {
            let (f, l) = if self.ignore_case {
                (filter.to_lowercase(), label.to_lowercase())
            } else {
                (filter.clone(), label.clone())
            };
            if f == l {
                return true;
            }
        }
        false
    }

    fn keep_by_name(&self, bookmark: &Bookmark) -> bool {
        let (filter, name) = if self.ignore_case {
            (
                self.get_filter().to_lowercase(),
                bookmark.name.to_lowercase(),
            )
        } else {
            (self.get_filter(), bookmark.name.clone())
        };
        name == filter || name.contains(&filter)
    }

    fn keep_by_id(&self, bookmark: &Bookmark) -> bool {
        bookmark
            .id
            .to_string()
            .starts_with(&self.filter.to_string())
    }

    fn get_filter(&self) -> String {
        if self.ignore_case {
            return self.filter.to_lowercase();
        };
        self.filter.clone()
    }
}

impl Filter<Bookmark> for BookmarkFilter {
    fn keep(&self, bookmark: &Bookmark) -> bool {
        match self.mode {
            Mode::Name => self.keep_by_name(bookmark),
            Mode::ID => self.keep_by_id(bookmark),
            Mode::Label => self.keep_by_label(bookmark),
        }
    }
}
