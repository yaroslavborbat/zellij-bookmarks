use crate::filters::Filter;

#[derive(Default)]
pub(crate) struct TabManager<T> {
    origin: Vec<T>,
    items: Vec<T>,
    selected: usize,
}

impl<T: Clone> TabManager<T> {
    pub(crate) fn new(items: Vec<T>) -> Self {
        TabManager {
            origin: items.clone(),
            items,
            selected: 0,
        }
    }

    pub(crate) fn select_down(&mut self) {
        if self.selected == self.items.len() - 1 {
            self.selected = 0;
            return;
        }
        self.selected += 1;
    }

    pub(crate) fn select_up(&mut self) {
        if self.selected == 0 {
            self.selected = self.items.len() - 1;
            return;
        }
        self.selected -= 1;
    }

    pub(crate) fn reset_selection(&mut self) {
        self.selected = 0;
    }

    pub(crate) fn get_selected(&self) -> Option<&T> {
        self.items.get(self.selected)
    }

    pub(crate) fn get_position(&self) -> usize {
        self.selected
    }

    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn with_filter(&mut self, f: Box<dyn Filter<T>>) {
        self.items = self
            .origin
            .iter()
            .filter(|item| f.keep(item))
            .cloned()
            .collect();
        self.reset_selection();
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.items.iter().enumerate()
    }
}
