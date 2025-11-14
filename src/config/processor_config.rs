use super::blox_config::{BloxConfig, BloxConfigTitleType};
use super::css_config::CssConfig;
use super::figure_config::FigureConfig;
use super::{CODE_BLOCK_KEYWORD, PREPROCESSOR_NAME};
use anyhow::{Context, Result};
use hex_color::HexColor;
use mdbook_preprocessor::PreprocessorContext;
use serde::Deserialize;
use std::collections::hash_map::{HashMap, Keys};
use std::fs;
use std::path::PathBuf;

#[derive(Eq, PartialEq, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    renderer: String,
    #[serde(deserialize_with = "CssConfig::deserialize")]
    css: CssConfig,
    defaults: ConfigDefaults,
    environments: HashMap<String, BloxConfig>,
    figure: FigureConfig,
}

impl Config {
    // GETTERS
    #[inline]
    pub fn renderer(&self) -> &str {
        &self.renderer
    }
    #[inline]
    pub fn css(&self) -> &CssConfig {
        &self.css
    }
    // #[inline]
    // fn config_defaults(&self) -> &ConfigDefaults {
    //     &self.defaults
    // }
    #[inline]
    pub fn get_environment_keys(&self) -> Keys<'_, String, BloxConfig> {
        self.environments.keys()
    }
    #[inline]
    pub fn get_env(&self, env: &str) -> Option<&BloxConfig> {
        self.environments.get(env).or_else(|| {
            tracing::error!("Environment not found: {env}");
            None
        })
    }
    #[inline]
    pub fn env_name(&self, env: &str) -> &str {
        self.get_env(env)
            .map(|e| e.name())
            .unwrap_or("ENVIRONMENT NOT DEFINED")
    }
    #[inline]
    pub fn env_color(&self, env: &str) -> &HexColor {
        self.get_env(env)
            .and_then(|e| e.color())
            .unwrap_or(self.defaults.color())
    }
    #[inline]
    pub fn env_prefix_number(&self, env: &str) -> bool {
        self.get_env(env)
            .and_then(|e| e.prefix_number())
            .unwrap_or(self.defaults.prefix_number())
    }
    #[inline]
    pub fn env_title_type(&self, env: &str) -> BloxConfigTitleType {
        self.get_env(env)
            .and_then(|e| e.title_type())
            .unwrap_or(self.defaults.title_type())
    }
    #[inline]
    pub fn fig_prefix_number(&self) -> bool {
        self.figure
            .prefix_number()
            .unwrap_or(self.defaults.prefix_number())
    }
    #[inline]
    pub fn fig_numbered(&self) -> bool {
        self.figure.numbered()
    }

    // ADDITIONAL GETTERS
    pub fn id(&self, env: &str) -> String {
        if !self.environments.contains_key(env) {
            tracing::error!("Environment not found: {env}");
        }

        format!("{CODE_BLOCK_KEYWORD}-{env}")
    }

    // CONSTRUCTORS
    pub fn from_file(file: &PathBuf) -> Result<Self> {
        MdbookConfig::from_file(file).map(|c| c.preprocessor.blox)
    }
    pub fn from_string(config: &str) -> Result<Self> {
        MdbookConfig::from_string(config).map(|c| c.preprocessor.blox)
    }
    pub fn from_context(ctx: &PreprocessorContext) -> Result<Self> {
        let table_key = format!("preprocessor.{PREPROCESSOR_NAME}");

        let Some(table) = ctx
            .config
            .get::<toml::Value>(&table_key)
            .context("book.toml failed to deserialize")?
        else {
            return Ok(Self::default());
        };

        // let value = toml::Value::Table(table.clone());
        // let mut config: Self = Self::deserialize(value)?;
        let mut config = Self::deserialize(table)?;
        config.renderer = ctx.renderer.clone();
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            renderer: "html".to_string(),
            css: CssConfig::default(),
            defaults: ConfigDefaults::default(),
            environments: HashMap::new(),
            figure: FigureConfig::default(),
        }
    }
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(default)]
struct ConfigDefaults {
    color: HexColor,
    /// If true, adds the section prefix to the numbers.
    prefix_number: bool,
    title_type: BloxConfigTitleType,
}

impl Default for ConfigDefaults {
    fn default() -> Self {
        Self {
            color: HexColor::from_u24(0xCE0037), // SLU Red
            prefix_number: true,
            title_type: BloxConfigTitleType::default(),
        }
    }
}

impl ConfigDefaults {
    #[inline]
    fn color(&self) -> &HexColor {
        &self.color
    }
    #[inline]
    fn prefix_number(&self) -> bool {
        self.prefix_number
    }
    #[inline]
    fn title_type(&self) -> BloxConfigTitleType {
        self.title_type
    }
}

#[derive(Default, Deserialize, Debug, Eq, PartialEq)]
#[serde(default)]
struct PreprocessorConfig {
    blox: Config,
}

#[derive(Default, Deserialize, Debug, Eq, PartialEq)]
#[serde(default)]
struct MdbookConfig {
    preprocessor: PreprocessorConfig,
}

impl MdbookConfig {
    fn from_file(file: &PathBuf) -> Result<Self> {
        let data = fs::read_to_string(file).context("Can't read configuration file")?;
        Self::from_string(&data)
    }
    fn from_string(config: &str) -> Result<Self> {
        let book_config: Self = toml::from_str(config).context("Invalid configuration file")?;
        Ok(book_config)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::config::blox_config::BloxConfigTitleType;

    use super::*;
    use pretty_assertions::assert_eq;

    const BLOX_CONFIG_STR: &'static str = r##"
[preprocessor.blox.defaults]
color = "#FF0000"

[preprocessor.blox.environments]
alert = {name = "Alert", color = "#00FF00", title_type = "numbered"}
exercise = {name = "Exercise"}
quote = {name = "Quote", color = "#CCCCCC", title_type = "minimal", prefix_number = false}
"##;

    pub fn default_test_config() -> Config {
        let mut config = Config::default();
        config.defaults.color = HexColor::from_u24(0xFF0000);
        config.defaults.title_type = BloxConfigTitleType::Numbered;
        config.environments.insert(
            "alert".to_string(),
            BloxConfig {
                name: "Alert".to_string(),
                color: Some(HexColor::from_u24(0x00FF00)),
                prefix_number: None,
                title_type: Some(BloxConfigTitleType::Numbered),
            },
        );
        config.environments.insert(
            "exercise".to_string(),
            BloxConfig {
                name: "Exercise".to_string(),
                color: None,
                prefix_number: None,
                title_type: None,
            },
        );
        config.environments.insert(
            "quote".to_string(),
            BloxConfig {
                name: "Quote".to_string(),
                color: Some(HexColor::from_u24(0xCCCCCC)),
                prefix_number: Some(false),
                title_type: Some(BloxConfigTitleType::Minimal),
            },
        );

        config
    }

    #[test]
    fn test_deserialize_from_string() -> Result<()> {
        let expected = default_test_config();
        let config = Config::from_string(BLOX_CONFIG_STR)?;

        assert_eq!(config, expected);

        // Check block
        assert_eq!(config.env_name("alert"), "Alert");
        assert_eq!(config.env_color("alert"), &HexColor::from_u24(0x00FF00));
        assert_eq!(config.env_prefix_number("alert"), true);
        assert_eq!(
            config.env_title_type("alert"),
            BloxConfigTitleType::Numbered
        );
        assert_eq!(config.env_color("exercise"), &HexColor::from_u24(0xFF0000));
        assert_eq!(
            config.env_title_type("exercise"),
            BloxConfigTitleType::Numbered
        );
        assert_eq!(config.env_prefix_number("quote"), false);
        assert_eq!(config.env_title_type("quote"), BloxConfigTitleType::Minimal);

        Ok(())
    }
}
