use anyhow::{Context, Result};
pub use regex::*;

/// TOML ASCII
pub const TASCII_MATCH: &str = "[[:alnum:]_-]";

// Captures ::: keyword re
pub fn regex_header(keyword: &str, re: &str, fence_len: usize) -> Result<Regex> {
    Regex::new(&format!(
        "^(?P<indent>[[:space:]]*)(?P<fence>:{{{fence_len},}})[[:space:]]*(?P<key>{keyword})[[:space:]]+{re}[[:space:]]*$",
    )).context("Couldn't parse header regex")
}

/// Captures :::
pub fn regex_footer(indent_len: usize, fence_len: usize) -> Result<Regex> {
    Regex::new(&format!(
        "^[[:space:]]{{{indent_len}}}(?P<fence>:{{{fence_len}}})[[:space:]]*$",
    ))
    .context("Couldn't parse footer regex")
}

/// Captures {{keyword-re: label}}
pub fn regex_handlebars(keyword: &str, refs: &str) -> Result<Regex> {
    Regex::new(
        &format!(
            "\\{{\\{{[[:space:]]*(?P<key>{keyword})-(?P<refs>{refs}):[[:space:]]*(?P<label>{TASCII_MATCH}+)[[:space:]]*\\}}\\}}"
        )
    ).context("Couldn't parse handlebars regex")
}

pub struct HandlebarCapture<'a> {
    pub keyword: &'a str,
    pub refs: &'a str,
    pub label: &'a str,
}
impl<'a> HandlebarCapture<'a> {
    #[inline]
    pub fn from_captures(caps: &Captures<'a>) -> Option<Self> {
        let keyword = caps.name("key").map(|s| s.as_str())?;
        let refs = caps.name("refs").map(|s| s.as_str())?;
        let label = caps.name("label").map(|s| s.as_str())?;
        Some(Self {
            keyword,
            refs,
            label,
        })
    }

    pub fn error(err: &str) -> String {
        format!("**{{{{ refernce error: {err} }}}}**",)
    }

    pub fn to_error(&self, err: &str) -> String {
        let HandlebarCapture {
            keyword,
            refs,
            label,
        } = self;
        tracing::error!("{{{{ {keyword}-{refs}: {label} }}}}: {err}",);
        format!("**{{{{ {keyword}-{refs}: {label}: {err} }}}}**",)
    }
}
