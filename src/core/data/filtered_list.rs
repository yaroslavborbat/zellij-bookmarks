use crate::core::filtering::Filter;

#[derive(Default)]
pub struct FilteredList<T> {
    origin: Vec<T>,
    items: Vec<Item<T>>,
    selected: usize,
}

pub struct Item<T> {
    pub indices: Vec<usize>,
    pub value: T,
}

impl<T: Clone> FilteredList<T> {
    pub fn new(items: Vec<T>) -> Self {
        FilteredList {
            origin: items.clone(),
            items: items
                .iter()
                .map(|i| Item {
                    value: i.clone(),
                    indices: Vec::new(),
                })
                .collect(),
            selected: 0,
        }
    }

    pub fn select_down(&mut self) {
        if self.selected == self.items.len() - 1 {
            self.selected = 0;
            return;
        }
        self.selected += 1;
    }

    pub fn select_up(&mut self) {
        if self.selected == 0 {
            self.selected = self.items.len() - 1;
            return;
        }
        self.selected -= 1;
    }

    pub fn reset_selection(&mut self) {
        self.selected = 0;
    }

    pub fn get_selected(&self) -> Option<&T> {
        if let Some(item) = self.items.get(self.selected) {
            return Some(&item.value);
        }
        None
    }

    pub fn get_position(&self) -> usize {
        self.selected
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn with_filter(&mut self, f: Box<dyn Filter<T>>) {
        let mut items = Vec::new();

        for item in self.origin.iter() {
            let (keep, i) = f.keep_indices(item);
            if keep {
                items.push(Item {
                    value: item.clone(),
                    indices: i,
                });
            }
        }

        self.items = items;
        self.reset_selection();
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &Item<T>)> {
        self.items.iter().enumerate()
    }
}
