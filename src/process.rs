use crate::config::Config;
use crate::parse::Blox;
use crate::render::BloxRender;
use anyhow::{Context, Result};
use mdbook::book::Chapter;
use pulldown_cmark::{CodeBlockKind::*, Event, Options, Parser, Tag};
use std::ops::Range;

#[derive(Debug)]
struct CodeBlockRanges {
    header: Range<usize>,
    content: Range<usize>,
    footer: Range<usize>,
    // start_fence_length: usize,
    // end_fence_length: usize,
}

impl CodeBlockRanges {
    fn new_from_block(content: &str, offset: usize) -> Result<Self> {
        let fence_character = content
            .chars()
            .next()
            .context("Couldn't find start of fenced block start")?;
        // let start_fence_length = content.chars().position(|c| c != fence_character).unwrap();
        let end_fence_length = content
            .chars()
            .rev()
            .position(|c| c != fence_character)
            .context("Couldn't find start of fenced block end")?;

        let content_start = content
            .find('\n')
            .context("Couldn't find end of fenced block start")?
            + offset;

        let content_end = content.len() - end_fence_length + offset;

        Ok(CodeBlockRanges {
            header: (offset..content_start),
            content: (content_start..content_end),
            footer: (content_end..(content.len() + offset)),
            // start_fence_length,
            // end_fence_length,
        })
    }

    #[allow(dead_code)]
    #[inline]
    fn h(&self) -> Range<usize> {
        self.header.clone()
    }
    #[allow(dead_code)]
    #[inline]
    fn c(&self) -> Range<usize> {
        self.content.clone()
    }
    #[allow(dead_code)]
    #[inline]
    fn f(&self) -> Range<usize> {
        self.footer.clone()
    }
}

pub fn process_section(chapter: &mut Chapter, config: &Config) -> Result<()> {
    log::debug!("Parsing chapter: {}", chapter.name);
    let opts = Options::empty();
    // let mut opts = Options::empty();
    // opts.insert(Options::ENABLE_TABLES);
    // opts.insert(Options::ENABLE_FOOTNOTES);
    // opts.insert(Options::ENABLE_STRIKETHROUGH);
    // opts.insert(Options::ENABLE_TASKLISTS);

    let events = Parser::new_ext(chapter.content.as_str(), opts);
    let mut blox_list: Vec<(CodeBlockRanges, Blox)> = vec![];

    let mut number: usize = 1;

    for (event, span) in events.into_offset_iter() {
        if let Event::Start(Tag::CodeBlock(Fenced(header))) = event.clone() {
            // Serialize header
            let Some(mut blox) = Blox::from_header(config, header.as_ref())? else {
                continue;
            };

            // Separate content from header and footer
            let ranges =
                CodeBlockRanges::new_from_block(&chapter.content[span.clone()], span.start)?;

            if blox.set_number(number) {
                number += 1;
            }

            blox_list.push((ranges, blox));
        }
    }

    if blox_list.is_empty() {
        return Ok(());
    }

    let mut new_content: String =
        String::with_capacity(chapter.content.len() + blox_list.len() * 200);
    let mut index: usize = 0;

    // Create new containing block
    // Insert back into chapter
    for (ranges, blox) in blox_list.iter() {
        new_content.push_str(&chapter.content[index..ranges.header.start]);
        new_content.push_str(&BloxRender::html(
            config,
            blox,
            &chapter.content[ranges.c()],
        ));

        index = ranges.footer.end;
    }

    new_content.push_str(&chapter.content[blox_list.last().unwrap().0.footer.end..]);
    chapter.content = new_content;

    Ok(())
}
