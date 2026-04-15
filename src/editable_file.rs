use crate::core::{IdGetter, LabelsGetter, NameGetter};
use std::path::Path;

#[derive(Default, Debug, Clone)]
pub(crate) struct EditableFile {
    pub id: usize,
    pub path: String,
}

impl EditableFile {
    pub(crate) fn managed_label(&self, filename: &str, dirname: &str) -> String {
        if self.path == filename {
            return "file::main".to_string();
        }

        let path = Path::new(&self.path);
        let relative_path = path
            .strip_prefix(dirname)
            .unwrap_or(path)
            .with_extension("");

        let suffix = relative_path
            .iter()
            .map(|segment| segment.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("::");

        format!("file::{}", suffix)
    }
}

impl IdGetter for EditableFile {
    fn get_id(&self) -> usize {
        self.id
    }
}

impl NameGetter for EditableFile {
    fn get_name(&self) -> String {
        self.path.clone()
    }
}

impl LabelsGetter for EditableFile {
    fn get_labels(&self) -> Vec<String> {
        panic!("unsupported")
    }
}

#[cfg(test)]
mod tests {
    use super::EditableFile;

    #[test]
    fn managed_label_uses_main_for_root_file() {
        let file = EditableFile {
            id: 1,
            path: ".zellij_bookmarks.yaml".to_string(),
        };

        assert_eq!(
            file.managed_label(".zellij_bookmarks.yaml", ".zellij-bookmarks.d"),
            "file::main"
        );
    }

    #[test]
    fn managed_label_uses_relative_path_without_extension() {
        let file = EditableFile {
            id: 2,
            path: ".zellij-bookmarks.d/team/kubernetes.yaml".to_string(),
        };

        assert_eq!(
            file.managed_label(".zellij_bookmarks.yaml", ".zellij-bookmarks.d"),
            "file::team::kubernetes"
        );
    }
}
