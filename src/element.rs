use std::{
    fmt::{self, Display, Formatter},
    fs,
    path::Path,
};

use toml::Value;

#[derive(Debug, Clone)]
pub struct Elements {
    pub locations: Vec<Element>,
    pub people: Vec<Element>,
    pub weapons: Vec<Element>,
    pub max_name_length: usize,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub state: ElementState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementIdentifier {
    pub element_type: ElementType,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]

pub enum ElementType {
    Weapon,
    Location,
    Person,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn process_action(&mut self, inp: &str) -> Option<Option<String>> {
        let mut chars = inp.chars();
        let section = match chars.next() {
            Some('l') => &mut self.locations,
            Some('p') => &mut self.people,
            Some('w') => &mut self.weapons,
            _ => return Some(None),
        };

        let index = chars
            .clone()
            .take_while(|x| x.is_ascii_digit())
            .collect::<String>();
        let index_len = index.len();
        let index = match index.parse::<usize>().ok().map(|x| x.saturating_sub(1)) {
            Some(x) if x >= section.len() => return Some(Some("Invalid index".to_owned())),
            Some(x) => x,
            None => return Some(None),
        };

        let new_state = match chars.nth(index_len) {
            Some('c') => ElementState::Confirmed,
            Some('d') => ElementState::Dismissed,
            Some('u') => ElementState::Unknown,
            _ => return Some(None),
        };

        section[index].state = new_state;
        None
    }

    pub fn get_state(&self, id: &ElementIdentifier) -> ElementState {
        let list = match id.element_type {
            ElementType::Location => &self.locations,
            ElementType::Person => &self.people,
            ElementType::Weapon => &self.weapons,
        };

        if id.index >= list.len() {
            return ElementState::Unknown;
        }

        list[id.index].state
    }

    pub fn set_state(&mut self, id: &ElementIdentifier, state: ElementState) {
        let list = match id.element_type {
            ElementType::Location => &mut self.locations,
            ElementType::Person => &mut self.people,
            ElementType::Weapon => &mut self.weapons,
        };

        if id.index >= list.len() {
            return;
        }

        list[id.index].state = state;
    }
}

#[derive(Debug)]
pub enum ProcesResult {
    Section,
    Index,
    Constraint,
}

impl Element {
    fn new(name: String) -> Self {
        Self {
            name,
            state: ElementState::Unknown,
        }
    }
}

impl Display for ProcesResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Section => write!(f, "Invalid section"),
            Self::Index => write!(f, "Invalid index"),
            Self::Constraint => write!(f, "Invalid constraint"),
        }
    }
}

impl Display for ElementIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let section = match self.element_type {
            ElementType::Location => "l",
            ElementType::Person => "p",
            ElementType::Weapon => "w",
        };

        write!(f, "{}{}", section, self.index + 1)
    }
}
