use super::leaf::Leaf;
use crate::config::blox_config::BloxConfigTitleType;
use crate::config::processor_config::Config;
use crate::parse::{sanitize_string_toml_ascii, to_toml_ascii};
use crate::process::regex::TASCII_MATCH;
use crate::render::Render;
use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Eq, PartialEq, Debug, Default)]
pub struct Blox {
    environment: String,
    content: String,
    label: Option<BloxLabel>,

    title: String,
    footer: String,
    number: String,

    // Defaultable
    title_type: BloxConfigTitleType,
}

impl Blox {
    // GETTERS
    #[inline]
    pub fn env(&self) -> &str {
        self.environment.as_str()
    }
    #[inline]
    pub fn content(&self) -> &str {
        self.content.as_str()
    }
    #[inline]
    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }
    #[inline]
    pub fn blox_label(&self) -> Option<&BloxLabel> {
        self.label.as_ref()
    }
    #[inline]
    pub fn label(&self) -> Option<&str> {
        Some(self.blox_label()?.label())
    }
    #[inline]
    pub fn title(&self) -> Option<&str> {
        match self.title_type {
            BloxConfigTitleType::Unscoped | BloxConfigTitleType::Titled => {
                Some(self.title.as_str())
            }
            BloxConfigTitleType::Numbered => {
                (!self.title.is_empty()).then_some(self.title.as_str())
            }
            _ => None,
        }
    }
    #[inline]
    pub fn footer(&self) -> Option<&str> {
        (!self.footer.is_empty()).then_some(self.footer.as_str())
    }
    #[inline]
    pub fn number(&self) -> Option<&str> {
        match self.title_type {
            BloxConfigTitleType::Numbered => Some(self.number.as_str()),
            _ => None,
        }
    }
    #[inline]
    pub fn title_type(&self) -> &BloxConfigTitleType {
        &self.title_type
    }

    // SETTERS
    pub fn set_number<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&str) -> Result<String>,
    {
        if self.title_type == BloxConfigTitleType::Numbered {
            self.number = f(self.env())?;
        }

        Ok(())
    }
    pub fn set_path(&mut self, path: &Path) -> Option<()> {
        let label = self.label.as_mut()?;
        label.path = path.to_path_buf();
        Some(())
    }

    // ADDITIONAL GETTERS
    pub fn header(&self, config: &Config) -> Option<String> {
        match self.title_type {
            BloxConfigTitleType::Unscoped => Some(self.ref_title()),
            BloxConfigTitleType::Titled => Some(self.ref_title_scoped(config)),
            BloxConfigTitleType::Numbered => {
                let mut s = self.ref_number(config);
                if !self.title.is_empty() {
                    s = format!("{s}: {title}", title = self.title);
                }
                Some(s)
            }
            _ => None,
        }
    }

    pub fn ref_default(&self, config: &Config) -> String {
        match self.title_type {
            BloxConfigTitleType::Unscoped => self.ref_title(),
            BloxConfigTitleType::Titled => self.ref_title_scoped(config),
            BloxConfigTitleType::Numbered => self.ref_number(config),
            _ => {
                tracing::warn!("Reference (default) cannot be created");
                "??".to_string()
            }
        }
    }
    pub fn ref_full(&self, config: &Config) -> String {
        match self.header(config) {
            Some(s) => s,
            _ => {
                tracing::warn!("Reference (full) cannot be created");
                "??".to_string()
            }
        }
    }
    pub fn ref_number(&self, config: &Config) -> String {
        match self.title_type {
            BloxConfigTitleType::Numbered => format!(
                "{env} {num}",
                env = config.env_name(self.env()),
                num = self.number,
            ),
            _ => {
                tracing::warn!("Numbered title cannot be created");
                "??".to_string()
            }
        }
    }
    pub fn ref_number_unscoped(&self) -> String {
        match self.title_type {
            BloxConfigTitleType::Numbered => self.number.clone(),
            _ => {
                tracing::warn!("Numbered title cannot be created");
                "??".to_string()
            }
        }
    }
    pub fn ref_title(&self) -> String {
        match self.title_type {
            BloxConfigTitleType::Unscoped
            | BloxConfigTitleType::Titled
            | BloxConfigTitleType::Numbered => self.title.clone(),
            _ => {
                tracing::warn!("Reference (title) cannot be created");
                "??".to_string()
            }
        }
    }
    pub fn ref_title_scoped(&self, config: &Config) -> String {
        match self.title_type {
            BloxConfigTitleType::Unscoped
            | BloxConfigTitleType::Titled
            | BloxConfigTitleType::Numbered => format!(
                "{env}: {title}",
                env = config.env_name(self.env()),
                title = self.title
            ),
            _ => {
                tracing::warn!("Reference (title scoped) cannot be created");
                "??".to_string()
            }
        }
    }

    pub fn id(&self, config: &Config) -> Option<String> {
        let label = self.label()?;
        Some(format!("{env}-{label}", env = config.id(self.env())))
    }

    // CONSTRUCTORS
    pub fn from_leaf(leaf: Leaf<'_>, config: &Config) -> Result<Self> {
        let Leaf::Blox {
            options, content, ..
        } = leaf
        else {
            bail!("Can only construct Blox from Leaf::Blox");
        };

        let re = Regex::new(&format!("(?P<env>{TASCII_MATCH}+)[[:space:]]*(?P<opts>.*)"))
            .context("Couldn't parse options regex")?
            .captures(&options)
            .ok_or(anyhow!("Invalid options string: couldn't parse options"))?;

        let env = re.name("env").map(|s| s.as_str()).unwrap_or("");

        let blox_header = re
            .name("opts")
            .map(|s| s.as_str())
            .and_then(|s| (!s.is_empty()).then_some(s))
            .map(|options| {
                let inline_toml = format!("options={{{options}}}");
                toml::from_str::<BloxHeaderWrapper>(&inline_toml)
                    .with_context(|| format!("Failed to parse blox options: {options}"))
                    .map(|v| v.options)
            })
            .unwrap_or_else(|| Ok(BloxHeader::default()))?;

        let blox = Blox {
            environment: env.to_string(),
            content: content.to_string(),
            label: BloxLabel::from_header(&blox_header),

            title: blox_header.title,
            footer: blox_header.footer,

            title_type: blox_header
                .title_type
                .unwrap_or_else(|| config.env_title_type(env)),

            ..Blox::default()
        };

        match blox.title_type {
            BloxConfigTitleType::Minimal => {
                if !blox.title.is_empty() {
                    tracing::warn!("Title will be ignored for title_type 'minimal'");
                }
            }
            BloxConfigTitleType::Unscoped | BloxConfigTitleType::Titled => {
                if blox.title.is_empty() {
                    tracing::error!("Title is not provided for a titled title_type");
                }
            }
            _ => {}
        };

        Ok(blox)
    }
}

impl Render for Blox {
    fn html(&self, config: &Config) -> String {
        let header = self
            .header(config)
            .map(|h| {
                format!(
                    r#"<div class="{header_class}">

{h}

</div>"#,
                    header_class = config.css().header()
                )
            })
            .unwrap_or_default();
        let footer = self
            .footer()
            .map(|f| {
                format!(
                    r#"<div class="{footer_class}">

{f}

</div>"#,
                    footer_class = config.css().footer(),
                )
            })
            .unwrap_or_default();
        let content = if self.content.trim().is_empty() {
            String::new()
        } else {
            format!(
                r##"<div class="{content_class}">

{content}

</div>"##,
                content_class = config.css().content(),
                content = self.content
            )
        };

        let id: String = self
            .id(config)
            .map(|id| format!(r#"id="{id}""#))
            .unwrap_or_default();

        format!(
            r##"<div {id} class="{block_class} {env_id}">{header}{content}{footer}</div>
"##,
            block_class = config.css().block(),
            env_id = config.id(self.env()),
        )
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct BloxLabel {
    label: String,
    defer_rendering: bool,
    path: PathBuf,
}

impl BloxLabel {
    // GETTERS
    #[inline]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }
    #[inline]
    pub fn defer_rendering(&self) -> bool {
        self.defer_rendering
    }
    #[inline]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    // CONSTRUCTORS
    fn with_label(label: &str) -> Self {
        Self {
            label: to_toml_ascii(label),
            ..Self::default()
        }
    }

    fn from_header(header: &BloxHeader) -> Option<Self> {
        if header.label.is_empty() {
            if header.defer_rendering {
                tracing::error!("Cannot defer rendering of a blox without a label");
            }

            return None;
        }

        let mut bl = Self::with_label(&header.label);
        bl.defer_rendering = header.defer_rendering;
        Some(bl)
    }
}

#[derive(Deserialize)]
struct BloxHeaderWrapper {
    options: BloxHeader,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct BloxHeader {
    title: String,
    footer: String,
    #[serde(deserialize_with = "sanitize_string_toml_ascii")]
    label: String,
    title_type: Option<BloxConfigTitleType>,
    defer_rendering: bool,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::{
        blox_config::BloxConfigTitleType, processor_config::test::default_test_config,
    };
    use pretty_assertions::assert_eq;
    use std::borrow::Cow;

    const CONTENT_STR: &'static str = "CONTENT";

    fn check_options(options: &str, expected: Blox) -> Result<()> {
        let leaf = Leaf::Blox {
            range: 0..0,
            content: Cow::Borrowed(CONTENT_STR),
            options: Cow::Borrowed(options),
        };

        let config = default_test_config();
        let blox = match Blox::from_leaf(leaf, &config) {
            Ok(c) => c,
            Err(e) => panic!("Couldn't create options: got \n{e}\nfrom\n{options}"),
        };

        assert_eq!(blox, expected);
        Ok(())
    }

    fn with_environment(env: &str, tt: BloxConfigTitleType) -> Blox {
        let mut blox = Blox::default();
        blox.environment = env.to_string();
        blox.content = CONTENT_STR.to_string();
        blox.title_type = tt;
        blox
    }

    #[test]
    fn test_construction() -> Result<()> {
        check_options(
            "alert",
            with_environment("alert", BloxConfigTitleType::Numbered),
        )?;

        check_options(
            "exercise",
            with_environment("exercise", BloxConfigTitleType::Numbered),
        )?;

        check_options(r#"alert title_type = "minimal", label = "warning-22" "#, {
            let mut blox = with_environment("alert", BloxConfigTitleType::Minimal);
            blox.label = Some(BloxLabel::with_label("warning-22"));
            blox
        })?;

        Ok(())
    }

    #[test]
    fn test_method() -> Result<()> {
        let config = default_test_config();

        let mut blox = with_environment("alert", BloxConfigTitleType::Numbered);
        blox.title = "Title".to_string();
        blox.number = "2".to_string();

        assert_eq!(blox.header(&config).as_deref(), Some("Alert 2: Title"));
        assert_eq!(blox.ref_title_scoped(&config).as_str(), "Alert: Title");

        Ok(())
    }
}
