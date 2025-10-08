use mdbook::book::Book;
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

/// A no-op preprocessor.
pub struct Nop;

impl Nop {
    pub fn new() -> Nop {
        Nop
    }
}

impl Preprocessor for Nop {
    fn name(&self) -> &str {
        "nop-preprocessor"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        // In testing we want to tell the preprocessor to blow up by setting a
        // particular config value
        match ctx
            .config
            .get_deserialized_opt("preprocessor.nop-preprocessor.blow-up")
        {
            Ok(Some(true)) => anyhow::bail!("Boom!!1!"),
            Ok(_) => {}
            Err(e) => anyhow::bail!("expect bool for blow-up: {e}"),
        }

        // we *are* a no-op preprocessor after all
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nop_preprocessor_run() {
        let input_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "nop": {}
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.4.21"
                },
                {
                    "items": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1\n",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ]
                }
            ]"##;
        let input_json = input_json.as_bytes();

        let (ctx, book) = mdbook_preprocessor::parse_input(input_json).unwrap();
        let expected_book = book.clone();
        let result = Nop::new().run(&ctx, book);
        assert!(result.is_ok());

        // The nop-preprocessor should not have made any changes to the book content.
        let actual_book = result.unwrap();
        assert_eq!(actual_book, expected_book);
    }
}
