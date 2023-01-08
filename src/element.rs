use std::{fs, path::Path};

use toml::Value;

#[derive(Debug)]
pub struct Elements {
    pub locations: Vec<Element>,
    pub people: Vec<Element>,
    pub weapons: Vec<Element>,
    pub max_name_length: usize,
}

#[derive(Debug)]
pub struct Element {
    pub name: String,
    pub state: ElementState,
}

#[derive(Debug, Clone, Copy)]
pub enum ElementState {
    Unknown,
    Confirmed,
    Dismissed,
}

impl Elements {
    pub fn load(path: impl AsRef<Path>) -> Option<Self> {
        let mut max_name_length = 0;
        let mut load_section = |section: &str, toml: &Value| -> Option<Vec<Element>> {
            let elements = toml
                .get(section)?
                .as_array()?
                .iter()
                .filter_map(|v| v.as_str())
                .map(|x| Element::new(x.to_owned()))
                .collect::<Vec<_>>();
            max_name_length =
                max_name_length.max(elements.iter().map(|x| x.name.len()).max().unwrap());
            Some(elements)
        };

        let raw = fs::read_to_string(path).ok()?;
        let toml = toml::from_str::<Value>(&raw).ok()?;

        Some(Self {
            locations: load_section("locations", &toml)?,
            people: load_section("people", &toml)?,
            weapons: load_section("weapons", &toml)?,
            max_name_length,
        })
    }
}

impl Element {
    fn new(name: String) -> Self {
        Self {
            name,
            state: ElementState::Unknown,
        }
    }
}

impl ElementState {
    pub fn as_char(&self) -> char {
        match self {
            Self::Unknown => '?',
            Self::Confirmed => 'C',
            Self::Dismissed => 'X',
        }
    }
}
