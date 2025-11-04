pub mod regex;
mod replace_refs;

use crate::blox::number_map::NumberMap;
use crate::blox::{collection::Collection, leaf::content_to_leafs};
use crate::config::CODE_BLOCK_KEYWORD;
use crate::config::processor_config::Config;
use anyhow::Result;
use mdbook::book::{Book, BookItem, Chapter};
use replace_refs::replace_refs;
use std::collections::HashMap;

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
    collection: Collection<'a>,
}

impl<'a> BloxProcessor<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            config,
            collection: Collection::default(),
        }
    }

    pub fn process(book: &mut Book, config: &'a Config) -> Result<HashMap<usize, String>> {
        let mut processor = Self::new(config);
        for (sec_id, chapter) in book_filter_iter(book) {
            processor.process_section(sec_id, &chapter.content)?;
        }

        processor.process_content(book)?;

        let mut new_content: HashMap<usize, String> = HashMap::new();

        for (sec_id, chapter) in book_filter_iter(book) {
            let path = chapter.path.as_ref();
            let mut content = processor.collection.section_to_string(config, sec_id);
            content = replace_refs(&processor.collection, content, config, path)?;
            new_content.insert(sec_id, content);
        }

        Ok(new_content)
    }

    fn process_section(&mut self, section_id: usize, content: &'a str) -> Result<()> {
        for leaf in content_to_leafs(CODE_BLOCK_KEYWORD, content)?.into_iter() {
            self.collection
                .push_raw_leaf(leaf, section_id, self.config)?;
        }
        Ok(())
    }

    fn process_content(&mut self, book: &Book) -> Result<()> {
        let mut number_map = NumberMap::new(self.config);

        for (section_id, chapter) in book_filter_iter(book) {
            number_map.reset(chapter.number.as_ref().map(|n| n.to_string()));
            let path = chapter.path.as_ref();

            self.collection
                .set_blox(section_id, &mut number_map, path)?;

            self.collection
                .process_figures(self.config, section_id, &mut number_map, path)?;
        }

        Ok(())
    }
}
