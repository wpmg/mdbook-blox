use crate::config::{CODE_BLOCK_KEYWORD, Config, to_toml_ascii};
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq)]
pub struct Blox {
    environment: String,
    options: BloxOptions,
}

impl Blox {
    pub fn new(environment: &str, options: BloxOptions) -> Self {
        Self {
            environment: environment.to_string(),
            options,
        }
    }

    /// Tries to parse `blox env [options]`
    pub fn from_header(config: &Config, header: &str) -> Result<Option<Self>> {
        let header = header.trim();

        // If the header doesn't start with `blox`, we exit early
        if !header.starts_with(CODE_BLOCK_KEYWORD) {
            return Ok(None);
        }

        let Some((keyword, rest)) = header.split_once(' ') else {
            return Ok(None);
        };

        // False alarm -- header must start with something like `bloxx`
        if keyword != CODE_BLOCK_KEYWORD {
            return Ok(None);
        }

        let (env, opts_str) = match rest.trim().split_once(' ') {
            Some((e, o)) => (e, Some(o.trim()).filter(|s| !s.is_empty())),
            None => (rest, None),
        };

        anyhow::ensure!(!env.is_empty(), "No blox environment specified");

        anyhow::ensure!(
            config.has_environment(env),
            "Blox environment not defined in book.toml"
        );

        let opts = match opts_str {
            Some(o) => BloxOptions::from_string(config, env, o)?,
            None => BloxOptions::from_code_block_options(config, env, CodeBlockOptions::default()),
        };

        Ok(Some(Blox::new(env, opts)))
    }

    pub fn set_number(&mut self, number: usize) -> bool {
        match self.options.number.as_mut() {
            Some(n) => {
                *n = number;
                true
            }
            None => false,
        }
    }

    #[inline]
    pub fn env(&self) -> &str {
        self.environment.as_str()
    }
    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.options.title.as_deref()
    }
    #[inline]
    pub fn footer(&self) -> Option<&str> {
        self.options.footer.as_deref()
    }
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.options.label.as_deref()
    }
    #[inline]
    pub fn defer_rendering(&self) -> bool {
        self.options.defer_rendering
    }
    #[inline]
    pub fn hide_name(&self, config: &Config) -> bool {
        self.options
            .hide_name
            .unwrap_or(config.hide_name(self.env()))
    }
    #[inline]
    pub fn hide_header(&self, config: &Config) -> bool {
        self.options
            .hide_header
            .unwrap_or(config.hide_header(self.env()))
    }
    #[inline]
    pub fn number(&self) -> Option<usize> {
        self.options.number
    }

    #[inline]
    pub fn group_str(&self, config: &Config) -> Option<String> {
        config.group_str(self.env()).ok()
    }
    #[inline]
    pub fn id_str(&self, config: &Config) -> Option<String> {
        let Some(group) = self.group_str(config) else {
            return None;
        };

        self.label().map(|label| format!("{group}-{label}"))
    }
    #[inline]
    pub fn number_str(&self, section_number: Option<&str>) -> Option<String> {
        self.number().map(|n| {
            let mut s = n.to_string();

            if let Some(sn) = section_number {
                s.insert_str(0, sn);
            }

            s
        })
    }
}

#[derive(Deserialize)]
struct CodeBlockOptionsWrapper {
    options: CodeBlockOptions,
}

#[derive(Default, Deserialize, Debug, PartialEq, Eq)]
struct CodeBlockOptions {
    /// A custom title
    #[serde(default)]
    title: Option<String>,
    /// A custom footer
    #[serde(default)]
    footer: Option<String>,
    /// A label(reference)
    #[serde(default)]
    label: Option<String>,
    /// If true, will defer the rendering of this block until explicitly stated
    #[serde(default)]
    defer_rendering: bool,

    // Defaultable
    /// Hiding the environment name
    #[serde(default)]
    hide_name: Option<bool>,
    /// Hide the environment header
    #[serde(default)]
    hide_header: Option<bool>,
    /// If true, it will have a number
    #[serde(default)]
    numbered: Option<bool>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct BloxOptions {
    pub title: Option<String>,
    pub footer: Option<String>,
    pub label: Option<String>,
    pub defer_rendering: bool,
    // Defaultable
    pub hide_name: Option<bool>,
    pub hide_header: Option<bool>,
    pub number: Option<usize>,
}

impl BloxOptions {
    fn from_string(config: &Config, environment: &str, options: &str) -> Result<Self> {
        let inline_toml = format!("options = {{ {} }}", options);
        let cb_opts: CodeBlockOptions =
            toml::from_str::<CodeBlockOptionsWrapper>(inline_toml.as_str())
                .context("Failed to parse blox options")?
                .options;

        let b_opts = BloxOptions::from_code_block_options(config, environment, cb_opts);

        Ok(b_opts)
    }
    fn from_code_block_options(
        config: &Config,
        environment: &str,
        options: CodeBlockOptions,
    ) -> Self {
        Self {
            title: options.title,
            footer: options.footer,
            label: options.label.as_deref().map(to_toml_ascii),
            defer_rendering: options.defer_rendering,
            // Defaultable
            hide_name: options.hide_name,
            hide_header: options.hide_header,
            number: options
                .numbered
                .unwrap_or(config.numbered(environment))
                .then_some(0),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::default_test_config;
    use pretty_assertions::assert_eq;

    fn check_options(options: &str, expected: Option<Blox>) -> Result<()> {
        let config = default_test_config();
        let blox = match Blox::from_header(&config, options) {
            Ok(c) => c,
            Err(e) => panic!("Couldn't create options: got \n{e}\nfrom\n{options}"),
        };

        assert_eq!(blox, expected);
        Ok(())
    }

    #[test]
    fn test() -> Result<()> {
        check_options(
            "blox alert",
            Some(Blox::new(
                "alert",
                BloxOptions {
                    title: None,
                    footer: None,
                    label: None,
                    defer_rendering: false,
                    hide_name: None,
                    hide_header: None,
                    number: None,
                },
            )),
        )?;

        check_options(
            "blox exercise",
            Some(Blox::new(
                "exercise",
                BloxOptions {
                    title: None,
                    footer: None,
                    label: None,
                    defer_rendering: false,
                    hide_name: None,
                    hide_header: None,
                    number: Some(0),
                },
            )),
        )?;

        check_options(
            r#"blox alert numbered = true, label = "warning-22" "#,
            Some(Blox::new(
                "alert",
                BloxOptions {
                    title: None,
                    footer: None,
                    label: Some("warning-22".to_string()),
                    defer_rendering: false,
                    hide_name: None,
                    hide_header: None,
                    number: Some(0),
                },
            )),
        )?;

        check_options("bloxx alert", None)?;
        check_options("block alert", None)?;

        Ok(())
    }
}
