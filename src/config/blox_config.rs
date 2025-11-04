use crate::parse::sanitize_string_toml_ascii;
use hex_color::HexColor;
use serde::Deserialize;

#[derive(Eq, PartialEq, Deserialize, Debug)]
#[serde(default)]
pub struct BloxConfig {
    #[serde(deserialize_with = "sanitize_string_toml_ascii")]
    pub(super) name: String,
    pub(super) color: Option<HexColor>,
    /// If true, adds the section prefix to the numbers.
    pub(super) prefix_number: Option<bool>,
    pub(super) title_type: Option<BloxConfigTitleType>,
}

impl BloxConfig {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[inline]
    pub fn color(&self) -> Option<&HexColor> {
        self.color.as_ref()
    }
    #[inline]
    pub fn prefix_number(&self) -> Option<bool> {
        self.prefix_number
    }
    #[inline]
    pub fn title_type(&self) -> Option<BloxConfigTitleType> {
        self.title_type
    }
}

impl Default for BloxConfig {
    fn default() -> Self {
        Self {
            name: "ENVIRONMENT_UNDEFINED".to_string(),
            color: None,
            prefix_number: None,
            title_type: None,
        }
    }
}

#[derive(Eq, PartialEq, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BloxConfigTitleType {
    /// Untitled
    Minimal,
    /// Hide environment string
    Unscoped,
    /// Show environment string, dont show number
    Titled,
    /// Show environment string, show number
    Numbered,
}

impl Default for BloxConfigTitleType {
    fn default() -> Self {
        Self::Numbered
    }
}
