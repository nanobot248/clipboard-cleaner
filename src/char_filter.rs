use crate::config::CharacterFilter;
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CharFilterComponent {
    Single(char),
    Range(char, char),
}

impl CharFilterComponent {
    pub fn matches(&self, ch: char) -> bool {
        match self {
            CharFilterComponent::Single(single_char) => {
                return ch == *single_char;
            }
            CharFilterComponent::Range(first, last) => {
                return *first <= ch && ch <= *last;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharFilter {
    components: Vec<CharFilterComponent>,
}

impl CharFilter {
    pub fn new(components: Vec<CharFilterComponent>) -> CharFilter {
        return CharFilter {
            components
        };
    }

    pub fn matches(&self, ch: char) -> bool {
        for component in self.components.iter() {
            if component.matches(ch) {
                return true;
            }
        }
        return false;
    }

    pub fn from_config(filter_config: &CharacterFilter) -> anyhow::Result<Self> {
        let mut components: Vec<CharFilterComponent> = Vec::new();
        for range in filter_config.ranges() {
            if range.single().is_some() {
                components.push(CharFilterComponent::Single(char::from_u32(range.single().unwrap()).unwrap()));
            } else if range.start().is_some() && range.end().is_some() {
                components.push(CharFilterComponent::Range(
                    char::from_u32(range.start().unwrap()).unwrap(),
                    char::from_u32(range.end().unwrap()).unwrap()
                ));
            } else {
                let text = format!("Character range must either be a single character or both start and end characters. Invalid character range: {:?}",
                    &range);
                return Err(anyhow::Error::msg(text));
            }
        }

        return Ok(CharFilter::new(components));
    }

}

// impl From<CharacterFilter> for CharFilter {
//     fn from(filter_config: CharacterFilter) -> anyhow::Result<Self> {
//         let mut components: Vec<CharFilterComponent> = Vec::new();
//         for range in filter_config.ranges() {
//             if range.single().is_some() {
//                 components.push(CharFilterComponent::Single(range.single().unwrap() as char));
//             } else if range.start().is_some() && range.end().is_some() {
//                 components.push(CharFilterComponent::Range(
//                     range.start().unwrap() as char,
//                     range.end().unwrap() as char
//                 ));
//             } else {
//                 let text = format!("Character range must either be a single character or both start and end characters. Invalid character range: {:?}",
//                     &range);
//                 return Err(anyhow::Error::msg(text.as_str()));
//             }
//         }
//
//         return Ok(CharFilter::new(components));
//     }
// }