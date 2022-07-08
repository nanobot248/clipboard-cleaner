use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    filters: HashMap<String, CharacterFilter>,
    profiles: Vec<TransformationProfile>,
    default_profile: Option<String>,
    gui_replacement_profile: Option<String>,
}

impl Config {
    pub fn filters(&self) -> &HashMap<String, CharacterFilter> {
        return &self.filters;
    }

    pub fn profiles(&self) -> &Vec<TransformationProfile> {
        return &self.profiles;
    }

    pub fn default_profile_name(&self) -> &Option<String> {
        return &self.default_profile;
    }

    pub fn default_profile(&self) -> Option<TransformationProfile> {
        if let Some(name) = self.default_profile.clone() {
            let profile = self.profiles.iter()
                .find(|profile| profile.name.as_str() == name.as_str())
                .map(|profile| profile.clone());
            return profile;
        }

        return None;
    }

    pub fn gui_replacement_profile_name(&self) -> &Option<String> {
        return &self.gui_replacement_profile;
    }

    pub fn gui_replacement_profile(&self) -> Option<TransformationProfile> {
        if let Some(name) = self.gui_replacement_profile.clone() {
            let profile = self.profiles.iter()
                .find(|profile| profile.name.as_str() == name.as_str())
                .map(|profile| profile.clone());
            return profile;
        }

        return None;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterFilter {
    ranges: Vec<CharacterRange>,
}

impl CharacterFilter {
    pub fn ranges(&self) -> &Vec<CharacterRange> {
        return &self.ranges;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterRange {
    single: Option<u32>,
    start: Option<u32>,
    end: Option<u32>,
}

impl CharacterRange {
    pub fn for_single(value: u32) -> CharacterRange {
        return CharacterRange {
            single: Some(value),
            start: None,
            end: None,
        };
    }

    pub fn for_range(start: u32, end: u32) -> CharacterRange {
        return CharacterRange {
            single: None,
            start: Some(start),
            end: Some(end),
        };
    }

    pub fn single(&self) -> Option<u32> {
        return self.single;
    }
    pub fn start(&self) -> Option<u32> {
        return self.start;
    }
    pub fn end(&self) -> Option<u32> {
        return self.end;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransformationAction {
    #[serde(rename = "remove")]
    Remove,
    #[serde(rename = "replace")]
    Replace(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WrappedFilter {
    #[serde(rename = "ref")]
    Reference(String),
    #[serde(rename = "filter")]
    Value(CharacterFilter),
}

impl WrappedFilter {
    pub fn character_filter(&self, config: &Config) -> Option<CharacterFilter> {
        match self {
            WrappedFilter::Reference(name) => {
                return config.filters.get(name.as_str()).map(|f| f.clone());
            }
            WrappedFilter::Value(filter) => {
                return Some(filter.clone());
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transformation {
    filters: Vec<WrappedFilter>,
    action: TransformationAction,
}

impl Transformation {
    pub fn new(filters: Vec<WrappedFilter>, action: TransformationAction) -> Transformation {
        return Transformation {
            filters,
            action
        };
    }

    pub fn filters(&self) -> &Vec<WrappedFilter> {
        return &self.filters;
    }

    pub fn action(&self) -> &TransformationAction {
        return &self.action;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformationProfile {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    transformations: Vec<Transformation>,
}

impl TransformationProfile {
    pub fn name(&self) -> &str {
        return &self.name;
    }
    pub fn display_name(&self) -> &Option<String> {
        return &self.display_name;
    }
    pub fn description(&self) -> &Option<String> {
        return &self.description;
    }
    pub fn transformations(&self) -> &Vec<Transformation> {
        return &self.transformations;
    }

    pub fn identity() -> TransformationProfile {
        return TransformationProfile {
            name: "identity".to_string(),
            display_name: Some("Identity transformation".to_string()),
            description: Some("Does not change the text.".to_string()),
            transformations: Vec::new()
        };
    }
}
