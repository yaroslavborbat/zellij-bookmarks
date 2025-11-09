use crate::core::filtering::traits::{Filter, IdGetter};

pub struct IdFilter {
    filter: String,
}

impl IdFilter {
    pub fn new(filter: String) -> Self {
        IdFilter { filter }
    }
}

impl<T: IdGetter> Filter<T> for IdFilter {
    fn keep(&self, getter: &T) -> bool {
        getter
            .get_id()
            .to_string()
            .starts_with(&self.filter.to_string())
    }
    fn keep_indices(&self, getter: &T) -> (bool, Vec<usize>) {
        (self.keep(getter), Vec::new())
    }
}
