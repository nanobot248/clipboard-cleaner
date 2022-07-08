use crate::char_filter::CharFilter;
use crate::config::TransformationProfile;
use crate::{Config, Transformation, TransformationAction};
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReplacementPattern {
    /// A string literal
    Literal(String),

    /// Each byte as an escaped, zero-padded hex value, e.g. `\x00\x00\x00\x41` or
    /// `\x00\x00\x00\x0D`
    EscapedHexBytes,

    /// The padded (4- or 8-digit, depending on value) unicode codepoint without any escaping.
    SimpleUnicodeCodepoint,

    /// The padded (4- or 8-digit, depending on value), escaped with the corresponding
    /// prefix `\u` for 4-digit, `\U` for 8-digit. e.g. `\uFFFD` or `\U000E0001`
    EscapedUnicode,

    /// The unicode codepoint as `U+hexcode`, e.g. `U+FFFD`. The hex value will not be padded.
    UplusUnicode,

    /// The hexadecimal unicode codepoint escaped in Rust debug style, e.g. `\u{fffd}`, `\u{d}`.
    RustDebugEscapedUnicode,

    /// Encode as HTML/XML entity `&#<decimal-number>;`
    Entity,

    /// The character will be inserted unmodified
    Identity
}

impl ReplacementPattern {
    pub fn replace(&self, ch: char) -> String {
        match self {
            ReplacementPattern::Literal(text) => {
                return text.clone();
            }
            ReplacementPattern::EscapedHexBytes => {
                let ch = ch as u32;
                let b1: u8 = ((ch >> 24) & 0xFF) as u8;
                let b2: u8 = ((ch >> 16) & 0xFF) as u8;
                let b3: u8 = ((ch >> 8) & 0xFF) as u8;
                let b4: u8 = (ch & 0xFF) as u8;
                return format!("\\x{:0>2x}\\x{:0>2x}\\x{:0>2x}\\x{:0>2x}", b1, b2, b3, b4);
            }
            ReplacementPattern::SimpleUnicodeCodepoint => {
                let ch = ch as u32;
                if ch > 0xFFFF {
                    return format!("{:0>8x}", ch);
                } else {
                    return format!("{:0>4x}", ch);
                }
            }
            ReplacementPattern::EscapedUnicode => {
                let ch = ch as u32;
                if ch > 0xFFFF {
                    return format!("\\U{:0>8x}", ch);
                } else {
                    return format!("\\u{:0>4x}", ch);
                }
            }
            ReplacementPattern::UplusUnicode => {
                let ch = ch as u32;
                return format!("U+{:x}", ch);
            }
            ReplacementPattern::RustDebugEscapedUnicode => {
                let ch = ch as u32;
                return format!("\\u{}{:x}{}", '{', ch, '}');
            }
            ReplacementPattern::Entity => {
                let ch = ch as u32;
                return format!("&#{};", ch);
            }
            ReplacementPattern::Identity => {
                return format!("{}", ch);
            }
        };
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SimpleTransformationAction {
    Remove,
    Replace(Vec<ReplacementPattern>),
}

impl SimpleTransformationAction {
    pub fn from_str(pattern: &str) -> anyhow::Result<Self> {
        let mut result: Vec<ReplacementPattern> = Vec::new();
        let mut special = false;
        let mut token = String::new();

        for ch in pattern.chars() {
            if (!special && ch != '{') || (special && ch != '}') {
                token.push(ch);
            } else if !special {
                special = true;
                result.push(ReplacementPattern::Literal(token.clone()));
                token.clear();
            } else if special {
                special = false;
                if token.as_str() == "ident" || token.as_str() == "char" {
                    result.push(ReplacementPattern::Identity);
                } else if token.as_str() == "hex-esc" {
                    result.push(ReplacementPattern::EscapedHexBytes);
                } else if token.as_str() == "uni-simple" {
                    result.push(ReplacementPattern::SimpleUnicodeCodepoint);
                } else if token.as_str() == "uni-esc" {
                    result.push(ReplacementPattern::EscapedUnicode);
                } else if token.as_str() == "uni-codepoint" {
                    result.push(ReplacementPattern::UplusUnicode);
                } else if token.as_str() == "rust" {
                    result.push(ReplacementPattern::RustDebugEscapedUnicode);
                } else if token.as_str() == "entity" {
                    result.push(ReplacementPattern::Entity);
                } else {
                    return Err(anyhow::Error::msg("Syntax error in replacement pattern."));
                }
                token.clear();
            }
        }
        if token.len() > 0 {
            result.push(ReplacementPattern::Literal(token));
        }

        return Ok(SimpleTransformationAction::Replace(result));
    }

    pub fn from_config(action_config: &TransformationAction) -> anyhow::Result<SimpleTransformationAction> {
        match action_config {
            TransformationAction::Remove => {
                return Ok(Self::Remove);
            }
            TransformationAction::Replace(pattern) => {
                let result = Self::from_str(pattern.as_str())?;
                println!("mapping {:?} to {:?}", pattern, &result);
                return Ok(result);
            }
        }
    }

    pub fn execute(&self, ch: char) -> Option<String> {
        match self {
            SimpleTransformationAction::Remove => {
                return None;
            }
            SimpleTransformationAction::Replace(pattern) => {
                let mut result = String::new();
                for replacer in pattern {
                    result += replacer.replace(ch).as_str();
                }
                return Some(result);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleTransformation {
    filters: Vec<CharFilter>,
    action: SimpleTransformationAction
}

impl SimpleTransformation {
    pub fn new(config: &Config, trafo: &Transformation) -> anyhow::Result<SimpleTransformation> {
        let mut filters: Vec<CharFilter> = Vec::new();
        for filter_config in trafo.filters() {
            let filter_config = filter_config.character_filter(config);
            if let Some(filter_config) = filter_config {
                let filter = CharFilter::from_config(&filter_config)?;
                filters.push(filter);
            }
        }

        let action = SimpleTransformationAction::from_config(trafo.action())?;

        return Ok(SimpleTransformation {
            filters,
            action
        });
    }

    pub fn execute(&self, text: &str) -> String {
        let mut output = String::new();
        for ch in text.chars() {
            let mut add_ch = true;
            for filter in self.filters.iter() {
                if filter.matches(ch) {
                    add_ch = false;
                    let transformed_char = self.action.execute(ch);
                    if let Some(transformed_char) = transformed_char {
                        output += transformed_char.as_str();
                    }
                    break;
                }
            }
            if add_ch {
                output.push(ch);
            }
        }
        return output;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextTransformation {
    transformations: Vec<SimpleTransformation>
}

impl TextTransformation {
    pub fn new(config: &Config, trafo_profile: &TransformationProfile) -> anyhow::Result<TextTransformation> {
        let mut transformations: Vec<SimpleTransformation> = Vec::new();
        for trafo_config in trafo_profile.transformations().iter() {
            let trafo = SimpleTransformation::new(config, trafo_config)?;
            transformations.push(trafo);
        }
        return Ok(TextTransformation {
            transformations
        });
    }

    pub fn execute(&self, text: &str) -> String {
        let mut current_text = text.to_string();
        for trafo in self.transformations.iter() {
            current_text = trafo.execute(current_text.as_str());
        }
        return current_text;
    }
}