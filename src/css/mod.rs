use crate::config::{CODE_BLOCK_KEYWORD, Config};
use anyhow::Result;

pub struct BloxCss;
impl BloxCss {
    pub fn block_class() -> String {
        format!("{CODE_BLOCK_KEYWORD}")
    }
    pub fn header_class() -> String {
        format!("{CODE_BLOCK_KEYWORD}-header")
    }
    pub fn content_class() -> String {
        format!("{CODE_BLOCK_KEYWORD}-content")
    }
    pub fn footer_class() -> String {
        format!("{CODE_BLOCK_KEYWORD}-footer")
    }

    pub fn base_css() -> String {
        // let block_class = BloxCss::block_class();
        // let header_class = BloxCss::header_class();
        // let content_class = BloxCss::content_class();
        // let footer_class = BloxCss::footer_class();

        format!(
            r####"
.{block_class} {{
  display: flow-root;
  margin-block: 1em;
  margin-inline: 0em;
  box-shadow: 0 0.2rem 1rem rgba(0, 0, 0, 0.05);
  border-inline-start-width: 0.4em;
  border-inline-start-style: solid;
  break-inside: avoid;
}}
.{block_class} > div {{
  padding-inline: 1em;
}}
.{block_class} > .{header_class} {{
  font-weight: bold;
  padding-block: 0.5em;
}}
.{block_class} > .{content_class} {{
  margin-block: 1em;
}}
.{block_class} > .{footer_class} {{
  font-style: italic;
  text-align: right;
  margin-block: 1em;
}}
@media print {{
  .{block_class} {{
    box-shadow: none;
  }}
}}
"####,
            block_class = BloxCss::block_class(),
            header_class = BloxCss::header_class(),
            content_class = BloxCss::content_class(),
            footer_class = BloxCss::footer_class(),
        )
    }
}

pub fn css_from_config(config: &Config) -> Result<String> {
    let mut css: String = BloxCss::base_css();

    for env in config.environments.keys() {
        css.push_str(css_from_environment(config, env)?.as_str());
    }

    Ok(css)
}

fn css_from_environment(config: &Config, env: &str) -> Result<String> {
    let block_class = BloxCss::block_class();
    let header_class = BloxCss::header_class();
    let group_str = config.group_str(env)?;
    let color = config.color(env).display_rgb();
    let tr_color = config.color(env).with_a(26).display_rgba();

    Ok(format!(
        r####"
.{block_class}.{group_str} {{
  border-color: {color};
}}
.{block_class}.{group_str} > .{header_class} {{
  background-color: {tr_color};
}}
"####
    ))
}
