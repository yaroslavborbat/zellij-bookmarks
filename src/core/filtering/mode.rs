use std::fmt;
use std::fmt::Formatter;

#[derive(Default, PartialEq, Copy, Clone, Debug)]
pub enum FilterMode {
    #[default]
    Name,
    ID,
    Label,
}

impl fmt::Display for FilterMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Name => "Name",
            Self::ID => "ID",
            Self::Label => "Label",
        };
        write!(f, "{}", name)
    }
}

impl FilterMode {
    pub fn switch_to(&self, mode: FilterMode) -> Self {
        if *self == mode {
            FilterMode::default()
        } else {
            mode
        }
    }
}
