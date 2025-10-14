use anyhow::{Context, Result};
use hex_color::HexColor;
use mdbook::preprocess::PreprocessorContext;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Preprocessor name
pub const PREPROCESSOR_NAME: &'static str = "blox";
pub const CODE_BLOCK_KEYWORD: &'static str = PREPROCESSOR_NAME;

pub fn default_css_file() -> String {
    format!("assets/{PREPROCESSOR_NAME}.css")
}

#[derive(Deserialize)]
pub struct MdbookConfig {
    #[serde(default)]
    preprocessor: PreprocessorsConfig,
}

#[derive(Default, Deserialize)]
pub struct PreprocessorsConfig {
    #[serde(default)]
    blox: Config,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    #[serde(deserialize_with = "sanitize_string_toml_ascii")]
    pub css: String,
    defaults: ConfigDefaults,
    #[serde(deserialize_with = "sanitize_map_keys_toml_ascii")]
    pub environments: HashMap<String, EnvironmentConfig>,
}

impl Config {
    pub fn from_context(ctx: &PreprocessorContext) -> Result<Self> {
        let table = ctx
            .config
            .get_preprocessor(PREPROCESSOR_NAME)
            .context("No configuration in book.toml")?;
        let value = toml::Value::Table(table.clone());
        let config: Self = Self::deserialize(value)?;

        Ok(config)
    }

    pub fn from_file(file: &PathBuf) -> Result<Self> {
        let data = fs::read_to_string(file).context("Can't read configuration file")?;
        let book_config: MdbookConfig =
            toml::from_str(&data).context("Invalid configuration file")?;
        Ok(book_config.preprocessor.blox)
    }

    #[inline]
    pub fn has_environment(&self, key: &str) -> bool {
        self.environments.contains_key(key)
    }
    #[inline]
    fn get(&self, key: &str) -> Option<&EnvironmentConfig> {
        self.environments.get(key).or_else(|| {
            log::error!("Environment not found: {key}");
            None
        })
    }
    #[inline]
    pub fn group_str(&self, key: &str) -> Result<String> {
        anyhow::ensure!(self.has_environment(key), "Environment does not exist");
        Ok(format!("{CODE_BLOCK_KEYWORD}-{key}"))
    }
    #[inline]
    pub fn name(&self, key: &str) -> &str {
        self.get(key)
            .map(|e| e.name.as_str())
            .unwrap_or("ENVIRONMENT")
    }
    #[inline]
    pub fn color(&self, key: &str) -> &HexColor {
        self.get(key)
            .and_then(|e| e.color.as_ref())
            .unwrap_or(&self.defaults.color)
    }
    pub fn prefix_number(&self, key: &str) -> bool {
        self.get(key)
            .and_then(|e| e.prefix_number)
            .unwrap_or(self.defaults.prefix_number)
    }
    #[inline]
    pub fn hide_name(&self, key: &str) -> bool {
        self.get(key)
            .and_then(|e| e.hide_name)
            .unwrap_or(self.defaults.hide_name)
    }
    #[inline]
    pub fn hide_header(&self, key: &str) -> bool {
        self.get(key)
            .and_then(|e| e.hide_name)
            .unwrap_or(self.defaults.hide_header)
    }
    #[inline]
    pub fn numbered(&self, key: &str) -> bool {
        self.get(key)
            .and_then(|e| e.numbered)
            .unwrap_or(self.defaults.numbered)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            css: default_css_file(),
            defaults: ConfigDefaults::default(),
            environments: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct ConfigDefaults {
    color: HexColor,
    prefix_number: bool,
    // BloxOptions
    hide_name: bool,
    hide_header: bool,
    numbered: bool,
}

impl Default for ConfigDefaults {
    fn default() -> Self {
        Self {
            color: HexColor::from_u24(0xCE0037), // SLU Red
            prefix_number: true,
            hide_name: false,
            hide_header: false,
            numbered: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct EnvironmentConfig {
    name: String,
    color: Option<HexColor>,
    prefix_number: Option<bool>,
    // BloxOptions
    hide_name: Option<bool>,
    hide_header: Option<bool>,
    numbered: Option<bool>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            name: "ENVIRONMENT UNDEFINED".to_string(),
            color: None,
            prefix_number: None,
            // BloxOptions
            hide_name: None,
            hide_header: None,
            numbered: None,
        }
    }
}

pub fn to_toml_ascii(string: &str) -> String {
    string
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

pub fn sanitize_string_toml_ascii<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    Ok(to_toml_ascii(s.as_str()))
}

pub fn sanitize_map_keys_toml_ascii<'de, D, T>(
    deserializer: D,
) -> std::result::Result<HashMap<String, T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let map: HashMap<String, T> = HashMap::deserialize(deserializer)?;
    Ok(map
        .into_iter()
        .map(|(k, v)| (to_toml_ascii(k.as_str()), v))
        .collect())
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    const CONFIG_STR: &'static str = r##"
[defaults]
color = "#FF0000"
numbered = true

[environments]
alert = {name = "Alert", color = "#00FF00", numbered = false}
exercise = {name = "Exercise"}
"##;

    pub fn default_test_config() -> Config {
        let mut config = Config::default();
        config.defaults.color = HexColor::from_u24(0xFF0000);
        config.environments.insert(
            "alert".to_string(),
            EnvironmentConfig {
                name: "Alert".to_string(),
                color: Some(HexColor::from_u24(0x00FF00)),
                prefix_number: None,
                hide_name: None,
                hide_header: None,
                numbered: Some(false),
            },
        );
        config.environments.insert(
            "exercise".to_string(),
            EnvironmentConfig {
                name: "Exercise".to_string(),
                color: None,
                prefix_number: None,
                hide_name: None,
                hide_header: None,
                numbered: None,
            },
        );

        config
    }

    #[test]
    fn test_deserialize_from_toml() -> Result<()> {
        let config: Config = toml::from_str(CONFIG_STR)?;
        let expected = default_test_config();

        assert_eq!(config, expected);

        // Check block
        assert_eq!(config.name("alert"), "Alert");
        assert_eq!(*config.color("alert"), HexColor::from_u24(0x00FF00));
        assert_eq!(config.numbered("alert"), false);
        assert_eq!(config.name("exercise"), "Exercise");
        assert_eq!(*config.color("exercise"), HexColor::from_u24(0xFF0000));
        assert_eq!(config.numbered("exercise"), true);

        Ok(())
    }
}
