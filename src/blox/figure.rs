use crate::config::FIGURE_BLOCK_KEYWORD;
use crate::config::processor_config::Config;
use crate::parse::sanitize_string_toml_ascii;
use crate::render::Render;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::PathBuf;

use super::leaf::Leaf;

// Figure: only to store labelled figures for ref. matching.
// Numbering as-we-go on parse.
#[derive(Default, Debug, Eq, PartialEq)]
pub struct Figure {
    src: String,
    label: String,
    alt: String,

    content: String,
    path: PathBuf,
    number: Option<String>,
}

impl Figure {
    // GETTERS
    #[inline]
    pub fn src(&self) -> &str {
        self.src.as_str()
    }
    #[inline]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }
    #[inline]
    pub fn alt(&self) -> &str {
        self.alt.as_str()
    }
    #[inline]
    pub fn content(&self) -> &str {
        self.content.as_str()
    }
    #[inline]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    #[inline]
    pub fn number(&self) -> Option<&str> {
        self.number.as_deref()
    }
    #[inline]
    pub fn id(&self) -> Option<String> {
        (!self.label().is_empty())
            .then(|| format!("{FIGURE_BLOCK_KEYWORD}-{label}", label = self.label()))
    }
    fn header(&self) -> Option<String> {
        self.number.as_ref().map(|n| format!("Figure {n}"))
    }
    pub fn ref_default(&self) -> String {
        self.header()
            .unwrap_or("Reference (default) cannot be created".to_string())
    }
    pub fn ref_number(&self) -> String {
        self.number
            .as_deref()
            .unwrap_or("Reference (number) cannot be created")
            .to_string()
    }

    // SETTERS
    pub fn set_number<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce() -> Result<String>,
    {
        if self.number.is_some() {
            self.number = Some(f()?);
        }

        Ok(())
    }
    pub fn set_path(&mut self, path: Option<&PathBuf>) -> Option<()> {
        self.path = path?.clone();
        Some(())
    }

    // CONSTRUCTOR
    fn with_src(src: String) -> Self {
        let mut fig = Self::default();
        fig.src = src;

        if fig.src.is_empty() {
            log::error!("blox-figure is missing src");
        }

        fig
    }

    pub fn from_leaf(leaf: Leaf<'_>, config: &Config) -> Result<Self> {
        let Leaf::Blox {
            options, content, ..
        } = leaf
        else {
            bail!("Can only construct Blox from RawLeaf::Blox");
        };

        let figure_header = {
            let inline_toml = format!("options={{{options}}}");
            toml::from_str::<FigureHeaderWrapper>(&inline_toml)
                .with_context(|| format!("Failed to parse blox-figure options: {options}"))
                .map(|v| v.options)
        }?;

        let mut fig = Self::with_src(figure_header.src);
        fig.label = figure_header.label;
        fig.content = content.to_string();

        if figure_header.numbered.unwrap_or(config.fig_numbered()) {
            fig.number = Some(String::new());
        }

        Ok(fig)
    }
}

impl Render for Figure {
    fn html(&self, config: &Config) -> String {
        let id = self
            .id()
            .map(|s| format!(r#"id="{s}""#))
            .unwrap_or_default();

        let mut caption = self
            .header()
            .map(|s| format!("**{s}:**"))
            .unwrap_or_default();

        if !caption.is_empty() || !self.content().trim().is_empty() {
            caption = format!(
                r##"<figcaption>

{caption}
{content}

</figcaption>"##,
                content = self.content()
            );
        }

        let alt = (!self.alt().is_empty())
            .then(|| format!(r#"alt="{alt}""#, alt = self.alt()))
            .unwrap_or_default();

        format!(
            r##"<figure {id} class="{figure_class}"><img src="{src}" {alt} />{caption}</figure>
"##,
            figure_class = config.css().figure(),
            src = self.src(),
        )
    }

    fn latex(&self, _config: &Config) -> String {
        let id = self
            .id()
            .map(|s| format!(r#"\label{{{s}}}"#))
            .unwrap_or(String::new());
        let caption = (!self.content().trim().is_empty())
            .then(|| {
                format!(
                    r##"\caption{{%
{content}
}}"##,
                    content = self.content()
                )
            })
            .unwrap_or_default();

        format!(
            r##"\begin{{figure}}[ht]\centering\includegraphics{{{src}}}{caption}{id}\end{{figure}}
"##,
            src = self.src(),
        )
    }
}

#[derive(Deserialize)]
struct FigureHeaderWrapper {
    options: FigureHeader,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct FigureHeader {
    src: String,
    #[serde(deserialize_with = "sanitize_string_toml_ascii")]
    label: String,
    alt: String,
    numbered: Option<bool>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::processor_config::test::default_test_config;
    use pretty_assertions::assert_eq;
    use std::borrow::Cow;

    #[test]
    pub fn test_from_leaf() -> Result<()> {
        let config = default_test_config();

        let leaf = Leaf::Blox {
            range: 0..0,
            content: Cow::Owned("hej".to_string()),
            options: Cow::Owned(r#"src="a.a""#.to_string()),
        };

        let mut expected = Figure::with_src("a.a".to_string());
        expected.content = "hej".to_string();
        expected.number = Some(String::new());

        assert_eq!(expected, Figure::from_leaf(leaf, &config)?);
        assert_eq!(expected.id(), None);

        Ok(())
    }
}
