use owo_colors::OwoColorize;
use zellij_tile::prelude::{print_text_with_coordinates, Text};

pub struct ErrorManager {
    error: Option<String>,
    crit_error: Option<String>,
}

impl ErrorManager {
    pub fn new() -> Self {
        ErrorManager {
            error: None,
            crit_error: None,
        }
    }

    pub fn handle_error(&mut self, error: String) {
        self.error = Some(error.clone());
        eprintln!("Error: {}", error);
    }

    pub fn handle_crit_error(&mut self, crit_error: String) {
        self.crit_error = Some(crit_error.clone());
        eprintln!("Critical Error: {}", crit_error);
    }

    pub fn render(&mut self) -> bool {
        if let Some(e) = self.crit_error.as_ref() {
            let text = Text::new(format!("ERROR: {}", e.red()));
            print_text_with_coordinates(text, 1, 1, None, None);
            return true;
        }
        if let Some(e) = self.error.take() {
            let text = Text::new(format!("ERROR: {}", e.red()));
            print_text_with_coordinates(text, 1, 1, None, None);
            return true;
        }
        false
    }
}
