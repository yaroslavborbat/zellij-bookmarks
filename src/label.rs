use crate::core::{IdGetter, LabelsGetter, NameGetter};

#[derive(Default, Debug, Clone)]
pub(crate) struct Label {
    pub id: usize,
    pub name: String,
}

impl Label {
    pub(crate) fn new(id: usize, name: String) -> Self {
        Self { id, name }
    }
}

impl NameGetter for Label {
    fn get_name(&self) -> String {
        self.name.to_string()
    }
}

impl IdGetter for Label {
    fn get_id(&self) -> usize {
        self.id
    }
}

impl LabelsGetter for Label {
    fn get_labels(&self) -> Vec<String> {
        panic!("unsupported")
    }
}
