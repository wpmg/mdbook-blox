pub mod regex;
mod replace_refs;

use crate::config::CODE_BLOCK_KEYWORD;
use crate::config::processor_config::Config;
use crate::models::collection::Collection;
use crate::models::leaf::content_to_leafs;
use anyhow::Result;
use mdbook_preprocessor::book::Book;
use replace_refs::replace_refs;

pub fn process(book: &mut Book, config: &Config) -> Result<()> {
    let mut collection = Collection::new(config);

    // Traverse and process the chapters into the collections
    for chapter in book.chapters() {
        if chapter.path.is_none() {
            continue;
        }

        for leaf in content_to_leafs(CODE_BLOCK_KEYWORD, &chapter.content)?.into_iter() {
            collection.push_raw_leaf(leaf, chapter, config)?;
        }
    }

    collection.process_collections()?;

    book.for_each_chapter_mut(|chapter| {
        let Some(ref path) = chapter.path else {
            return;
        };
        let Some(path_str) = path.to_str() else {
            return;
        };

        let mut content = collection.section_to_string(path_str);
        content = replace_refs(&collection, content, config, path).unwrap_or_else(|e| {
            tracing::error!(error = %e, "Failed while replacing refs");
            String::default()
        });
        chapter.content = content;
    });

    Ok(())
}
