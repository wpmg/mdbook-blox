use serde::Deserialize;

#[derive(Eq, PartialEq, Deserialize, Debug)]
#[serde(default)]
pub struct FigureConfig {
    prefix_number: Option<bool>,
    numbered: bool,
}

impl FigureConfig {
    #[inline]
    pub fn prefix_number(&self) -> Option<bool> {
        self.prefix_number
    }
    #[inline]
    pub fn numbered(&self) -> bool {
        self.numbered
    }
}

impl Default for FigureConfig {
    fn default() -> Self {
        Self {
            prefix_number: None,
            numbered: true,
        }
    }
}
