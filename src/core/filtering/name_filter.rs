use crate::core::filtering::traits::{Filter, NameGetter};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub struct NameFilter {
    filter: String,
    ignore_case: bool,
}

impl NameFilter {
    pub fn new(filter: String, ignore_case: bool) -> Self {
        NameFilter {
            filter,
            ignore_case,
        }
    }
}

impl<T: NameGetter> Filter<T> for NameFilter {
    fn keep(&self, getter: &T) -> bool {
        if self.filter.is_empty() {
            return true;
        }
        let (filter, name) = if self.ignore_case {
            (self.filter.to_lowercase(), getter.get_name().to_lowercase())
        } else {
            (self.filter.clone(), getter.get_name().clone())
        };
        name == filter || name.contains(&filter)
    }

    fn keep_indices(&self, getter: &T) -> (bool, Vec<usize>) {
        let keep = self.keep(getter);
        if keep {
            let indices = (0..self.filter.len()).collect();
            return (true, indices);
        }
        (false, Vec::new())
    }
}

pub struct NameFuzzyFilter {
    filter: String,
    matcher: SkimMatcherV2,
}

impl NameFuzzyFilter {
    pub fn new(filter: String, _: bool) -> Self {
        let matcher = SkimMatcherV2::default();
        NameFuzzyFilter { filter, matcher }
    }
}

impl<T: NameGetter> Filter<T> for NameFuzzyFilter {
    fn keep(&self, getter: &T) -> bool {
        if self.filter.is_empty() {
            return true;
        }

        let score = self
            .matcher
            .fuzzy_match(getter.get_name().as_str(), self.filter.as_str());

        if let Some(s) = score {
            return s.is_positive();
        }

        false
    }

    fn keep_indices(&self, getter: &T) -> (bool, Vec<usize>) {
        if self.filter.is_empty() {
            return (true, Vec::new());
        }

        if let Some((score, indices)) = self
            .matcher
            .fuzzy_indices(getter.get_name().as_str(), self.filter.as_str())
        {
            return (score.is_positive(), indices);
        };

        (false, Vec::new())
    }
}
