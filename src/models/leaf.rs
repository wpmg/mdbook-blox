use crate::process::regex::{
    HandlebarCapture, Regex, regex_footer, regex_handlebars, regex_header,
};
use anyhow::Result;
use std::borrow::Cow;
use std::ops::Range;

#[derive(Eq, PartialEq, Debug)]
pub enum Leaf<'a> {
    None,
    Text {
        range: Range<usize>,
        content: Cow<'a, str>,
    },
    Blox {
        range: Range<usize>,
        content: Cow<'a, str>,
        options: Cow<'a, str>,
    },
    BloxReference {
        range: Range<usize>,
        label: String,
    },
}

impl<'a> Leaf<'a> {
    #[inline]
    pub fn ok(self) -> Option<Self> {
        match self {
            Self::None => None,
            _ => Some(self),
        }
    }
    #[inline]
    pub fn start(&self) -> usize {
        match self {
            Self::Text { range, .. }
            | Self::Blox { range, .. }
            | Self::BloxReference { range, .. } => range.start,
            _ => usize::MAX,
        }
    }
    #[inline]
    pub fn end(&self) -> usize {
        match self {
            Self::Text { range, .. }
            | Self::Blox { range, .. }
            | Self::BloxReference { range, .. } => range.end,
            _ => usize::MAX,
        }
    }
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
    #[inline]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    #[inline]
    pub fn to_text(range: Range<usize>, content: &'a str) -> Leaf<'a> {
        if range.is_empty() {
            return Leaf::None;
        }

        Leaf::Text {
            range,
            content: Cow::Borrowed(content),
        }
    }
    #[inline]
    pub fn to_blox(range: Range<usize>, content: &'a str, options: &'a str) -> Leaf<'a> {
        Leaf::Blox {
            range,
            content: Cow::Borrowed(content),
            options: Cow::Borrowed(options),
        }
    }
    #[inline]
    pub fn to_blox_reference(range: Range<usize>, label: &str) -> Leaf<'a> {
        Leaf::BloxReference {
            range,
            label: label.to_string(),
        }
    }
}

pub fn content_to_leafs<'a>(keyword: &str, content: &'a str) -> Result<Vec<Leaf<'a>>> {
    let header_regex = regex_header(keyword, "(?P<options>.*?)", 3)?;
    let render_regex = regex_handlebars(keyword, "render")?;

    let mut leafs: Vec<Leaf> = Vec::new();
    let mut offset = 0;

    // Identify blox
    // Store start of header row and data from header
    let mut fence_header: Option<FenceHeader> = None;

    for line in content.split_inclusive('\n') {
        if let Some(ref header) = fence_header {
            // if fence_header is some, we're looking for the closing fence
            if let Some(item) = header.close_block(content, line, offset).ok() {
                leafs.push(item);
                fence_header = None;
            }
        } else {
            // if fence_header is none, we're looking for
            //  - an opening fence
            //  - a refence to a block
            if let Some(fh) = FenceHeader::capture_header(&header_regex, line, offset) {
                let start = leafs.last().map(|item| item.end()).unwrap_or(0);
                let range = start..offset;
                leafs.push(Leaf::to_text(range.clone(), &content[range]));

                fence_header = Some(fh);
            } else {
                render_regex.captures_iter(line).for_each(|caps| {
                    let Some(hb_caps) = HandlebarCapture::from_captures(&caps) else {
                        return;
                    };

                    let start = leafs.last().map(|item| item.end()).unwrap_or(0);

                    // Close the containing text block
                    let range = start..(offset + caps.get_match().start());
                    leafs.push(Leaf::to_text(range.clone(), &content[range.clone()]));
                    let range = range.end..(offset + caps.get_match().end());
                    leafs.push(Leaf::to_blox_reference(range, hb_caps.label));
                });
            }
        }

        // Add offset at end
        offset += line.len();
    }

    // Handle dangling text
    let start = leafs.last().map(|item| item.end()).unwrap_or(0);
    let range = start..content.len();
    leafs.push(Leaf::to_text(range.clone(), &content[range]));

    Ok(leafs)
}

pub fn content_to_leafs_excl_reference<'a>(
    keyword: &str,
    content: &'a str,
) -> Result<Vec<Leaf<'a>>> {
    let header_regex = regex_header(keyword, "(?P<options>.*?)", 3)?;

    let mut leafs: Vec<Leaf> = Vec::new();
    let mut offset = 0;

    // Identify blox
    // Store start of header row and data from header
    let mut fence_header: Option<FenceHeader> = None;

    for line in content.split_inclusive('\n') {
        if let Some(ref header) = fence_header {
            // if fence_header is some, we're looking for the closing fence
            if let Some(item) = header.close_block(content, line, offset).ok() {
                leafs.push(item);
                fence_header = None;
            }
        } else {
            // if fence_header is none, we're looking for
            //  - an opening fence
            if let Some(fh) = FenceHeader::capture_header(&header_regex, line, offset) {
                let start = leafs.last().map(|item| item.end()).unwrap_or(0);
                let range = start..offset;
                leafs.push(Leaf::to_text(range.clone(), &content[range]));

                fence_header = Some(fh);
            }
        }

        // Add offset at end
        offset += line.len();
    }

    // Handle dangling text
    let start = leafs.last().map(|item| item.end()).unwrap_or(0);
    let range = start..content.len();
    leafs.push(Leaf::to_text(range.clone(), &content[range]));

    Ok(leafs)
}

struct FenceHeader<'a> {
    line_idx: usize,
    #[allow(dead_code)]
    indent_len: usize,
    options: &'a str,
    content_idx: usize,
    block_end_regex: Regex,
}

impl<'a> FenceHeader<'a> {
    pub fn capture_header(re: &Regex, line: &'a str, offset: usize) -> Option<Self> {
        let caps = re.captures(line)?;

        let fence_len = caps.name("fence").map(|s| s.as_str().len()).unwrap_or(0);
        if fence_len < 3 {
            return None;
        }

        let indent_len = caps.name("indent").map(|s| s.as_str().len()).unwrap_or(0);

        Some(Self {
            line_idx: offset,
            indent_len,
            options: caps.name("options").map(|s| s.as_str()).unwrap_or(""),
            content_idx: offset + line.len(),
            block_end_regex: regex_footer(indent_len, fence_len).ok()?,
        })
    }

    pub fn close_block(&self, chapter: &'a str, line: &str, offset: usize) -> Leaf<'a> {
        if !self.block_end_regex.is_match(line) {
            return Leaf::None;
        }

        let block_range = self.line_idx..(offset + line.len());
        let content_range = self.content_idx..offset;
        let content = &chapter[content_range];

        Leaf::to_blox(block_range, content, self.options)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::FIGURE_BLOCK_KEYWORD;
    use pretty_assertions::assert_eq;
    use std::borrow::Cow;

    const TEST_CONTENT: &'static str = r#"
text1
...
text1end

::: bloxfig src="a.a" label="b"
caption
:::

text2
...
text2end
"#;

    #[test]
    fn test_content_to_leafs_excl_reference() -> Result<()> {
        let expected_content: Cow<'_, str> = Cow::Owned("caption\n".to_string());
        let expected_options: Cow<'_, str> = Cow::Owned(r#"src="a.a" label="b""#.to_string());

        let leaf_vec = content_to_leafs_excl_reference(FIGURE_BLOCK_KEYWORD, TEST_CONTENT)?;
        let Leaf::Blox {
            ref content,
            ref options,
            ..
        } = leaf_vec[1]
        else {
            anyhow::bail!("leaf[1] is not Blox");
        };

        assert_eq!(expected_content, *content);
        assert_eq!(expected_options, *options);

        Ok(())
    }
}
