use crate::config::{CODE_BLOCK_KEYWORD, Config, to_toml_ascii};
use anyhow::{Context, Result};
use pathdiff::diff_paths;
use serde::Deserialize;
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Blox<'a> {
    /// Must be a key that matches environments in config
    pub environment: String,
    pub path: Option<PathBuf>,
    pub content: Cow<'a, str>,

    pub defer_rendering: bool,

    pub title: Option<String>,
    pub footer: Option<String>,
    pub label: Option<String>,
    pub number: Option<String>,

    // Defaultable
    pub hide_name: bool,
    pub hide_header: bool,
}

impl<'a> PartialEq for Blox<'a> {
    fn eq(&self, other: &Blox) -> bool {
        self.environment == other.environment
            && self.title == other.title
            && self.footer == other.footer
            && self.label == other.label
            && self.number == other.number
            && self.defer_rendering == other.defer_rendering
            && self.hide_name == other.hide_name
            && self.hide_header == other.hide_header
    }
}

impl<'a> Blox<'a> {
    #[cfg(test)]
    pub fn new(environment: &str) -> Self {
        let mut blox = Self::default();
        blox.environment = environment.to_string();
        blox
    }

    /// Tries to parse `blox env [options]`
    pub fn parse(config: &Config, content: &'a str, header: &str) -> Result<Option<Self>> {
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

        // Parse CodeBlockOptions from header
        let options = match opts_str {
            Some(o) => CodeBlockOptions::from_string(o)?,
            None => CodeBlockOptions::default(),
        };

        let hide_header = options.hide_header.unwrap_or(config.hide_header(env));
        // Hide name if header is hidden
        let hide_name = hide_header || options.hide_name.unwrap_or(config.hide_name(env));
        // Only numbered if name is not hidden and is numbered
        let number = (!hide_name && options.numbered.unwrap_or(config.numbered(env)))
            .then_some(String::new());

        let opts = Self {
            environment: env.to_string(),

            content: extract_content(content)?,
            path: None,

            title: options.title,
            footer: options.footer,
            label: options.label.as_deref().map(to_toml_ascii),
            defer_rendering: options.defer_rendering,

            // Defaultable
            hide_header,
            hide_name,
            number,
        };

        Ok(Some(opts))
    }

    #[inline]
    pub fn env(&self) -> &str {
        self.environment.as_str()
    }
    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    #[inline]
    pub fn title_numbered(&self, config: &Config) -> Option<String> {
        let num = self.number()?;
        let env_name = config.name(self.env());
        Some(format!("{env_name} {num}"))
    }
    #[inline]
    pub fn title_env(&self, config: &Config) -> Option<String> {
        let title = self.title()?;
        let mut s = config.name(self.env()).to_string();
        s.push_str(&format!(": {title}"));
        Some(s)
    }
    #[inline]
    pub fn title_full(&self, config: &Config) -> String {
        let mut s = config.name(self.env()).to_string();

        if let Some(n) = self.number() {
            s.push_str(&format!(" {n}"));
        }

        if let Some(title) = self.title() {
            s.push_str(&format!(": {title}"));
        }

        s
    }
    #[inline]
    pub fn title_auto(&self, config: &Config) -> Option<String> {
        if self.hide_name {
            return self.title().map(|s| s.to_owned());
        }

        Some(self.title_full(config))
    }
    #[inline]
    pub fn footer(&self) -> Option<&str> {
        self.footer.as_deref()
    }
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    #[inline]
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }
    #[inline]
    pub fn rel_path(&self, base: &PathBuf) -> Option<String> {
        let path = self.path()?;

        if path == base {
            return Some(String::new());
        }

        let mut base = base.clone();
        base.pop();
        Some(
            diff_paths(path, base)
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap(),
        )
    }
    #[inline]
    pub fn defer_rendering(&self) -> bool {
        self.defer_rendering
    }
    #[inline]
    pub fn number(&self) -> Option<&str> {
        self.number.as_deref()
    }
    #[inline]
    pub fn set_number(&mut self, number: usize, section_number: Option<&str>) -> bool {
        if self.number.is_none() {
            return false;
        }

        let mut s = number.to_string();

        if let Some(sn) = section_number {
            s.insert_str(0, sn);
        }

        self.number = Some(s);
        return true;
    }
    // #[inline]
    // pub fn hide_name(&self) -> bool {
    //     self.hide_name.clone()
    // }
    #[inline]
    pub fn hide_header(&self) -> bool {
        self.hide_header.clone()
    }

    #[inline]
    pub fn group_str(&self, config: &Config) -> Option<String> {
        config.group_str(self.env()).ok()
    }
    #[inline]
    pub fn id_str(&self, config: &Config) -> Option<String> {
        let group = self.group_str(config)?;
        self.label().map(|label| format!("{group}-{label}"))
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
    /// Hiding the environment name (if true, forces numbered to be hidden)
    #[serde(default)]
    hide_name: Option<bool>,
    /// Hide the environment header (if true, forces environment name to be hidden)
    #[serde(default)]
    hide_header: Option<bool>,
    /// If true, it will have a number
    #[serde(default)]
    numbered: Option<bool>,
}

impl CodeBlockOptions {
    fn from_string(options: &str) -> Result<Self> {
        let inline_toml = format!("options = {{ {options} }}");
        let cb_opts: CodeBlockOptions =
            toml::from_str::<CodeBlockOptionsWrapper>(inline_toml.as_str())
                .with_context(|| format!("Failed to parse blox options: {options}"))?
                .options;

        Ok(cb_opts)
    }
}

fn extract_content<'a>(content: &'a str) -> Result<Cow<'a, str>> {
    let fence_character = content
        .chars()
        .next()
        .context("Couldn't find start of fenced block start")?;
    let end_fence_length = content
        .chars()
        .rev()
        .position(|c| c != fence_character)
        .context("Couldn't find start of fenced block end")?;
    let content_start = content
        .find('\n')
        .context("Couldn't find end of fenced block start")?;
    let content_end = content.len() - end_fence_length;

    Ok(Cow::Borrowed(&content[content_start..content_end]))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::default_test_config;
    use pretty_assertions::assert_eq;

    const CONTENT_STR: &'static str = "\nCONTENT\n";

    fn check_options(options: &str, expected: Option<Blox>) -> Result<()> {
        let block_content = format!(r#"```{options}{CONTENT_STR}```"#);

        let config = default_test_config();
        let blox = match Blox::parse(&config, &block_content, options) {
            Ok(c) => c,
            Err(e) => panic!("Couldn't create options: got \n{e}\nfrom\n{options}"),
        };

        assert_eq!(blox, expected);
        Ok(())
    }

    #[test]
    fn test_construction() -> Result<()> {
        check_options(
            "blox alert",
            Some({
                let mut blox = Blox::new("alert");
                blox.content = Cow::Borrowed(CONTENT_STR);
                blox
            }),
        )?;

        check_options(
            "blox exercise",
            Some({
                let mut blox = Blox::new("exercise");
                blox.content = Cow::Borrowed(CONTENT_STR);
                blox.number = Some(String::new());
                blox
            }),
        )?;

        check_options(
            r#"blox alert numbered = true, label = "warning-22" "#,
            Some({
                let mut blox = Blox::new("alert");
                blox.content = Cow::Borrowed(CONTENT_STR);
                blox.label = Some("warning-22".to_string());
                blox.number = Some(String::new());
                blox
            }),
        )?;

        check_options("bloxx alert", None)?;
        check_options("block alert", None)?;

        Ok(())
    }

    #[test]
    fn test_method() -> Result<()> {
        let config = default_test_config();

        let mut blox = Blox::new("alert");
        blox.title = Some("Title".to_string());

        assert_eq!(blox.title_auto(&config).as_deref(), Some("Alert: Title"));

        blox.hide_name = true;
        assert_eq!(blox.title_auto(&config).as_deref(), Some("Title"));

        Ok(())
    }
}
