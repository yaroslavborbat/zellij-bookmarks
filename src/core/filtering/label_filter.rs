use crate::core::filtering::traits::{Filter, LabelsGetter};

pub struct LabelFilter {
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

impl<T: LabelsGetter> Filter<T> for LabelFilter {
    fn keep(&self, getter: &T) -> bool {
        if self.filter.is_empty() {
            return true;
        }
        for label in getter.get_labels().iter() {
            let (f, l) = if self.ignore_case {
                (self.filter.to_lowercase(), label.to_lowercase())
            } else {
                (self.filter.clone(), label.clone())
            };
            if f == l {
                return true;
            }
        }
        false
    }
    fn keep_indices(&self, getter: &T) -> (bool, Vec<usize>) {
        (self.keep(getter), Vec::new())
    }
}
