pub mod config;
pub mod models;
mod parse;
mod process;
mod render;

use crate::config::PREPROCESSOR_NAME;
pub use crate::config::processor_config::Config;
use anyhow::Result;
use mdbook_preprocessor::book::Book;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use process::process;
use tracing::debug_span;

/// A blox preprocessor
#[derive(Default)]
pub struct BloxPreProcessor;

impl BloxPreProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl Preprocessor for BloxPreProcessor {
    fn name(&self) -> &str {
        PREPROCESSOR_NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let span = debug_span!("config").entered();
        let config = Config::from_context(ctx)?;
        span.exit();

        let span = debug_span!("processing").entered();
        process(&mut book, &config)?;
        span.exit();

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        match renderer {
            "html" | "latex" => Ok(true),
            _ => Ok(false),
        }
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
