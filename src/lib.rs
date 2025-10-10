pub mod config;
pub mod css;
mod parse;
mod process;
mod render;

use crate::config::Config;
pub use crate::config::PREPROCESSOR_NAME;
use crate::process::process_book;
use anyhow::Result;
use mdbook::book::Book;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

/// A no-op preprocessor.
pub struct BloxProcessor;

impl BloxProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl Preprocessor for BloxProcessor {
    fn name(&self) -> &str {
        PREPROCESSOR_NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let config = Config::from_context(ctx)?;
        process_book(&mut book, &config)?;
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn nop_preprocessor_run() {
//         let input_json = r##"[
//                 {
//                     "root": "/path/to/book",
//                     "config": {
//                         "book": {
//                             "authors": ["AUTHOR"],
//                             "language": "en",
//                             "src": "src",
//                             "title": "TITLE"
//                         },
//                         "preprocessor": {
//                             "nop": {}
//                         }
//                     },
//                     "renderer": "html",
//                     "mdbook_version": "0.4.21"
//                 },
//                 {
//                     "items": [
//                         {
//                             "Chapter": {
//                                 "name": "Chapter 1",
//                                 "content": "# Chapter 1\n",
//                                 "number": [1],
//                                 "sub_items": [],
//                                 "path": "chapter_1.md",
//                                 "source_path": "chapter_1.md",
//                                 "parent_names": []
//                             }
//                         }
//                     ]
//                 }
//             ]"##;
//         let input_json = input_json.as_bytes();

//         let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();
//         let expected_book = book.clone();
//         let result = Nop::new().run(&ctx, book);
//         assert!(result.is_ok());

//         // The nop-preprocessor should not have made any changes to the book content.
//         let actual_book = result.unwrap();
//         assert_eq!(actual_book, expected_book);
//     }
// }
