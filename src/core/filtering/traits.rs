pub trait Filter<T> {
    fn keep(&self, t: &T) -> bool;
    fn keep_indices(&self, getter: &T) -> (bool, Vec<usize>);
}

pub trait NameGetter {
    fn get_name(&self) -> String;
}

pub trait IdGetter {
    fn get_id(&self) -> usize;
}

pub trait LabelsGetter {
    fn get_labels(&self) -> Vec<String>;
}
