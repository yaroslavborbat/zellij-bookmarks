use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use zellij_tile::prelude::*;

pub fn parse_key_info(binding: &String) -> Result<Keybinding, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = binding.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(format!("Invalid keybinding format: {}", binding).into());
    }
    let modifier = KeyModifier::from_str(parts[0])?;
    let key_char = parts[1].chars().next().ok_or("Missing key character")?;
    Ok(Keybinding::new(modifier, key_char))
}

#[derive(Clone)]
pub struct Keybinding {
    key_with_modifier: KeyWithModifier,
}

impl Keybinding {
    pub fn new(modifier: KeyModifier, key: char) -> Self {
        Self {
            key_with_modifier: KeyWithModifier::new_with_modifiers(
                BareKey::Char(key),
                BTreeSet::from([modifier]),
            ),
        }
    }

    pub(crate) fn matches(&self, key: &KeyWithModifier) -> bool {
        self.key_with_modifier.eq(key)
    }
}

impl fmt::Display for Keybinding {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.key_with_modifier.fmt(f)
    }
}
