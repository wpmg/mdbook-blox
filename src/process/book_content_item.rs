use crate::config::Config;
use crate::parse::Blox;
use crate::render::BloxRender;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug)]
pub enum BookContentItem<'a> {
    AnonymousBlox(usize),
    LabelledBlox(String),
    Other(Cow<'a, str>),
}

impl<'a> BookContentItem<'a> {
    pub fn new_anonymous(id: usize) -> Self {
        Self::AnonymousBlox(id)
    }
    pub fn new_labelled(label: &str) -> Self {
        Self::LabelledBlox(label.to_string())
    }
    pub fn new_other(content: &'a str) -> Option<Self> {
        if content.is_empty() {
            return None;
        }

        Some(Self::Other(Cow::Borrowed(&content)))
    }
    pub fn new_other_empty() -> Self {
        Self::Other(Cow::default())
    }

    pub fn to_html(
        &self,
        config: &Config,
        anon_list: &Vec<Blox>,
        label_list: &HashMap<String, Blox>,
    ) -> Cow<'a, str> {
        match self {
            Self::AnonymousBlox(id) => {
                let s: Cow<'a, str> = anon_list
                    .get(*id)
                    .map(|b| Cow::Owned(BloxRender::html(config, b)))
                    .unwrap_or_default();
                s
            }
            Self::LabelledBlox(label) => label_list
                .get(label)
                .map(|b| Cow::Owned(BloxRender::html(config, b)))
                .unwrap_or_default(),
            Self::Other(content) => content.clone(),
        }
    }
}
