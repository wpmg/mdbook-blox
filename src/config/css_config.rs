use super::processor_config::Config;
use crate::config::{CODE_BLOCK_KEYWORD, FIGURE_BLOCK_KEYWORD, PREPROCESSOR_NAME};
use serde::{Deserialize, Deserializer};

#[derive(Eq, PartialEq, Debug)]
pub struct CssConfig {
    file_path: String,
    block: String,
    header: String,
    content: String,
    footer: String,
    figure: String,
}

impl Default for CssConfig {
    fn default() -> Self {
        Self {
            file_path: format!("assets/{PREPROCESSOR_NAME}.css"),
            block: format!("{CODE_BLOCK_KEYWORD}"),
            header: format!("{CODE_BLOCK_KEYWORD}-header"),
            content: format!("{CODE_BLOCK_KEYWORD}-content"),
            footer: format!("{CODE_BLOCK_KEYWORD}-footer"),
            figure: format!("{FIGURE_BLOCK_KEYWORD}"),
        }
    }
}

impl CssConfig {
    // CONSTRUCTORS
    pub fn new(file: String) -> Self {
        let mut config = Self::default();

        if !file.is_empty() {
            config.file_path = file;
        }

        config
    }
    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let file = String::deserialize(deserializer)?;
        Ok(Self::new(file))
    }

    // GETTERS
    #[inline]
    pub fn file(&self) -> &str {
        &self.file_path
    }
    #[inline]
    pub fn block(&self) -> &str {
        &self.block
    }
    #[inline]
    pub fn header(&self) -> &str {
        &self.header
    }
    #[inline]
    pub fn content(&self) -> &str {
        &self.content
    }
    #[inline]
    pub fn footer(&self) -> &str {
        &self.footer
    }
    #[inline]
    pub fn figure(&self) -> &str {
        &self.figure
    }

    pub fn base_css(&self) -> String {
        format!(
            r####"
.{figure} {{
  margin-block: 1em;
}}
.{figure} > img {{

}}
.{figure} > figcaption {{

}}

.{block} {{
  display: flow-root;
  margin-block: 1em;
  margin-inline: 0em;
  box-shadow: 0 0.2rem 1rem rgba(0, 0, 0, 0.05);
  border-inline-start-width: 0.4em;
  border-inline-start-style: solid;
  break-inside: avoid;
}}
.{block} > div {{
  padding-inline: 1em;
}}
.{block} > .{header} {{
  display: flow-root;
  font-weight: bold;
}}
.{block} > .{content} {{
  margin-block: 1em;
}}
.{block} > .{footer} {{
  display: flow-root;
  font-style: italic;
  text-align: right;
}}
.{block} > .{header} > p, .{block} > .{footer} > p {{
  margin-block: 0.6em;
}}
@media print {{
  .{block} {{
    box-shadow: none;
  }}
}}
"####,
            block = self.block,
            header = self.header,
            content = self.content,
            footer = self.footer,
            figure = self.figure,
        )
    }

    fn environment_css(&self, config: &Config, env: &str) -> String {
        let group_str = config.id(env);
        let color = config.env_color(env).display_rgb();
        let tr_color = config.env_color(env).with_a(26).display_rgba();

        format!(
            r####"
.{block}.{group_str} {{
  border-color: {color};
}}
.{block}.{group_str} > .{header} {{
  background-color: {tr_color};
}}
"####,
            block = self.block,
            header = self.header,
        )
    }

    pub fn css_string(&self, config: &Config) -> String {
        let mut css: String = self.base_css();
        css.push_str(
            config
                .get_environment_keys()
                .map(|env| self.environment_css(config, env))
                .collect::<String>()
                .as_str(),
        );

        css
    }
}
