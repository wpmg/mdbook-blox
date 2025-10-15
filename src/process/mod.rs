mod book_content_item;
mod number_map;

use crate::config::Config;
use crate::parse::Blox;
use anyhow::{Context, Result};
use book_content_item::BookContentItem;
use mdbook::book::{Book, BookItem, Chapter};
use number_map::NumberMap;
use pulldown_cmark::{CodeBlockKind::*, Event, Parser, Tag};
use regex::{Captures, Regex};
use std::{collections::HashMap, ops::Range};

pub fn book_filter_iter(book: &Book) -> impl Iterator<Item = (usize, &Chapter)> {
    book.sections
        .iter()
        .enumerate()
        .filter_map(|(sec_id, item)| match item {
            BookItem::Chapter(chapter) => Some((sec_id, chapter)),
            _ => None,
        })
}

pub fn book_filter_iter_mut(book: &mut Book) -> impl Iterator<Item = (usize, &mut Chapter)> {
    book.sections
        .iter_mut()
        .enumerate()
        .filter_map(|(sec_id, item)| match item {
            BookItem::Chapter(chapter) => Some((sec_id, chapter)),
            _ => None,
        })
}

pub struct BloxProcessor<'a> {
    config: &'a Config,
    anonymous_blox: Vec<Blox<'a>>,
    labelled_blox: HashMap<String, Blox<'a>>,
    section_items: HashMap<usize, Vec<BookContentItem<'a>>>,
}

impl<'a> BloxProcessor<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            config,
            anonymous_blox: Vec::new(),
            labelled_blox: HashMap::new(),
            section_items: HashMap::new(),
        }
    }

    pub fn process(book: &mut Book, config: &'a Config) -> Result<HashMap<usize, String>> {
        let mut processor = Self::new(config);
        for (sec_id, chapter) in book_filter_iter(book) {
            processor.process_section(sec_id, &chapter.content)?;
        }

        processor.number_items(book)?;

        let mut new_content: HashMap<usize, String> = HashMap::new();

        for (sec_id, chapter) in book_filter_iter(book) {
            let content_string = processor.stringify_section(sec_id)?;
            let content_string = processor.replace_refs(content_string, chapter)?;
            new_content.insert(sec_id, content_string);
        }

        Ok(new_content)
    }

    fn process_section(&mut self, section_id: usize, chapter: &'a str) -> Result<()> {
        let cmark_opts = pulldown_cmark::Options::empty();
        // opts.insert(Options::ENABLE_TABLES);
        // opts.insert(Options::ENABLE_FOOTNOTES);
        // opts.insert(Options::ENABLE_STRIKETHROUGH);
        // opts.insert(Options::ENABLE_TASKLISTS);

        let mut items: Vec<(Range<usize>, BookContentItem)> = Vec::new();
        let events = Parser::new_ext(&chapter, cmark_opts);

        for (event, span) in events.into_offset_iter() {
            if let Event::Start(Tag::CodeBlock(Fenced(header))) = event.clone() {
                // If so, check if it is a blox-block
                let Some(blox) = Blox::parse(self.config, &chapter[span.clone()], header.as_ref())?
                else {
                    // Otherwise, store the content and move on
                    if let Some(bc) = BookContentItem::new_other(&chapter[span.clone()]) {
                        items.push((span, bc));
                    }
                    continue;
                };

                // Store labelled and anonymous blox separately
                if let Some(label) = blox.label.clone() {
                    // Deferred blox is not pushed
                    if !blox.defer_rendering() {
                        let content = BookContentItem::new_labelled(&label);
                        items.push((span, content));
                    } else {
                        items.push((span, BookContentItem::new_other_empty()));
                    }

                    self.labelled_blox.insert(label, blox);
                } else {
                    let content = BookContentItem::new_anonymous(self.anonymous_blox.len());
                    items.push((span, content));
                    self.anonymous_blox.push(blox);
                }
            }
        }

        let render_regex_pattern =
            r#"\{\{[[:space:]]*blox-render:[[:space:]]*(?P<label>[[:alnum:]_-]+)[[:space:]]*\}\}"#;
        let render_regex = Regex::new(render_regex_pattern).unwrap();
        let mut other_items: Vec<(Range<usize>, BookContentItem)> = Vec::new();
        let mut last = 0;

        // Special item for loop to work
        items.push((
            chapter.len()..chapter.len(),
            BookContentItem::new_other_empty(),
        ));

        for (span, _) in items.iter() {
            // Any other type of content might be a deferred blox-block
            for caps in render_regex.captures_iter(&chapter[last..span.start]) {
                let c_start = caps.get_match().start() + last;
                if let Some(bc) = BookContentItem::new_other(&chapter[last..c_start]) {
                    other_items.push((last..c_start, bc));
                }

                let c_end = caps.get_match().end() + last;
                if let Some(l) = caps.name("label") {
                    other_items.push((c_start..c_end, BookContentItem::new_labelled(l.as_str())));
                }

                last = c_end;
            }

            if let Some(bc) = BookContentItem::new_other(&chapter[last..span.start]) {
                other_items.push((last..span.start, bc));
            }

            last = span.end;
        }

        items.append(&mut other_items);
        items.sort_by(|a, b| a.0.start.cmp(&b.0.start));

        let items: Vec<BookContentItem> = items
            .into_iter()
            .filter(|(span, _)| !span.is_empty())
            .map(|item| item.1)
            .collect();

        self.section_items.insert(section_id, items);

        Ok(())
    }

    fn number_items(&mut self, book: &Book) -> Result<()> {
        let mut number_map = NumberMap::new(self.config);

        for (section_id, chapter) in book_filter_iter(book) {
            let chapter_number = chapter.number.as_ref().map(|n| n.to_string());

            let Some(items) = self.section_items.get_mut(&section_id) else {
                continue;
            };

            // Fix numbering
            for book_content in items.iter_mut() {
                let Some(blox) = (match book_content {
                    BookContentItem::AnonymousBlox(id) => self.anonymous_blox.get_mut(*id),
                    BookContentItem::LabelledBlox(s) => self.labelled_blox.get_mut(s),
                    _ => None,
                }) else {
                    continue;
                };

                number_map.set_blox(blox, chapter_number.as_deref())?;

                if blox.label().is_some() {
                    if blox.path().is_some() {
                        log::warn!("Multiple paths to blox: {}", blox.label().unwrap());
                    }

                    blox.path = chapter.path.clone();
                }
            }

            number_map.reset(self.config);
        }

        Ok(())
    }

    fn stringify_section(&self, section_id: usize) -> Result<String> {
        let items = self
            .section_items
            .get(&section_id)
            .context("Section id not found")?;
        let new_content: String = items
            .iter()
            .map(|item| item.to_html(self.config, &self.anonymous_blox, &self.labelled_blox))
            .collect::<Vec<_>>()
            .concat();

        Ok(new_content)
    }

    fn replace_refs(&self, content: String, chapter: &Chapter) -> Result<String> {
        // Can match "ref" here with, say, "tref" or similar, if multiple ref types is wanted
        let regex_pattern = r#"\{\{[[:space:]]*blox-(?P<ref>[ltnfTN]?ref):[[:space:]]*(?P<label>[[:alnum:]_-]+)[[:space:]]*\}\}"#;
        let regex = Regex::new(regex_pattern).context("Could not create regex")?;

        let new_content = regex
            .replace_all(&content, |caps: &Captures| {
                let Some(label) = caps.name("label").map(|l| l.as_str()) else {
                    return replace_refs_error("Regex match error", "ref", "error");
                };
                let Some(ref_type) = caps.name("ref").map(|r| r.as_str()) else {
                    return replace_refs_error("Unknown blox ref", "ref", label);
                };

                let Some(blox) = self.labelled_blox.get(label) else {
                    return replace_refs_error("Unknown blox ref", ref_type, label);
                };

                let Some(mut path) = chapter.path.as_ref().and_then(|p| blox.rel_path(p)) else {
                    return replace_refs_error("Failed to get path to blox", ref_type, label);
                };

                path.push_str(
                    &blox
                        .id_str(self.config)
                        .map(|s| format!("#{s}"))
                        .unwrap_or_default(),
                );

                match ref_type {
                    // Give title
                    "Tref" => blox.title().map(|s| s.to_string()).unwrap_or_else(|| {
                        replace_refs_error("Blox does not have a title", ref_type, label)
                    }),
                    // Give number
                    "Nref" => blox.number().map(|s| s.to_string()).unwrap_or_else(|| {
                        replace_refs_error("Blox does not have a number", ref_type, label)
                    }),
                    // Give link
                    "lref" => path,
                    // Provide linked environment-title
                    "tref" => blox
                        .title_env(self.config)
                        .map(|s| markdown_link(&s, &path))
                        .unwrap_or_else(|| {
                            replace_refs_error("Blox does not have a title", ref_type, label)
                        }),
                    // Provide linked environment-number
                    "nref" => blox
                        .title_numbered(self.config)
                        .map(|s| markdown_link(&s, &path))
                        .unwrap_or_else(|| {
                            replace_refs_error("Blox does not have a number", ref_type, label)
                        }),
                    // Provide linked environment-number-title
                    "fref" => markdown_link(&blox.title_full(self.config), &path),
                    // Provide environment-number, or environment-title if no number
                    _ => blox
                        .title_auto(self.config)
                        .map(|s| markdown_link(&s, &path))
                        .unwrap_or_else(|| {
                            replace_refs_error("Blox does not have a title", ref_type, label)
                        }),
                }
            })
            .to_string();

        Ok(new_content)
    }
}

fn replace_refs_error(label: &str, ref_type: &str, err: &str) -> String {
    log::warn!("{err}: {label}");
    format!("**[??blox-{ref_type}: {label}??]**")
}

fn markdown_link(text: &str, link: &str) -> String {
    format!("[{text}]({link})")
}
