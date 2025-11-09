use crate::core::filtering::id_filter::IdFilter;
use crate::core::filtering::label_filter::LabelFilter;
use crate::core::filtering::mode::FilterMode;
use crate::core::filtering::name_filter::{NameFilter, NameFuzzyFilter};
use crate::core::filtering::traits::{Filter, IdGetter, LabelsGetter, NameGetter};

pub struct GenericFilter {
    mode: FilterMode,
    name_filter: NameFilter,
    name_fuzzy_filter: NameFuzzyFilter,
    id_filter: IdFilter,
    label_filter: LabelFilter,
    fuzzy: bool,
}

impl GenericFilter {
    pub fn new(mode: FilterMode, filter: String, ignore_case: bool, fuzzy: bool) -> Self {
        GenericFilter {
            mode,
            name_filter: NameFilter::new(filter.clone(), ignore_case),
            name_fuzzy_filter: NameFuzzyFilter::new(filter.clone(), ignore_case),
            id_filter: IdFilter::new(filter.clone()),
            label_filter: LabelFilter::new(filter, ignore_case),
            fuzzy,
        }
    }
}

impl<T: NameGetter + IdGetter + LabelsGetter> Filter<T> for GenericFilter {
    fn keep(&self, getter: &T) -> bool {
        match self.mode {
            FilterMode::Name => {
                if self.fuzzy {
                    return self.name_fuzzy_filter.keep(getter);
                }
                self.name_filter.keep(getter)
            }
            FilterMode::ID => self.id_filter.keep(getter),
            FilterMode::Label => self.label_filter.keep(getter),
        }
    }

    fn keep_indices(&self, getter: &T) -> (bool, Vec<usize>) {
        match self.mode {
            FilterMode::Name => {
                if self.fuzzy {
                    return self.name_fuzzy_filter.keep_indices(getter);
                }
                self.name_filter.keep_indices(getter)
            }
            FilterMode::ID => self.id_filter.keep_indices(getter),
            FilterMode::Label => self.label_filter.keep_indices(getter),
        }
    }
}
