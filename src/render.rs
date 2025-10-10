use crate::config::Config;
use crate::css::BloxCss;
use crate::parse::Blox;

pub struct BloxRender;

impl BloxRender {
    // Returns None if header should be hidden
    fn header(config: &Config, blox: &Blox, section_number: Option<&str>) -> Option<String> {
        if blox.hide_header(config) {
            return None;
        }

        let mut pfx_title: Vec<String> = vec![];

        if !blox.hide_name(config) {
            if let Ok(name) = config.name(blox.env()) {
                pfx_title.push(name.to_string());
            }
        }

        if let Some(n) = blox.number_str(section_number) {
            pfx_title.push(n);
        }

        // Implicit hide_header if name is hidden, numbering is removed and no title is provided
        if pfx_title.is_empty() {
            return blox.title().map(|s| s.to_string());
        }

        let mut header: String = pfx_title.join(" ");

        if let Some(title) = blox.title() {
            header.push_str(&format!(": {title}"));
        }

        Some(header)
    }

    pub fn html(
        config: &Config,
        blox: &Blox,
        content: &str,
        section_number: Option<&str>,
    ) -> String {
        let block_class = BloxCss::block_class();
        let content_class = BloxCss::content_class();

        let header = Self::header(config, blox, section_number)
            .map(|h| {
                format!(
                    r#"<div class="{header_class}">{h}</div>"#,
                    header_class = BloxCss::header_class()
                )
            })
            .unwrap_or_default();
        let footer = blox
            .footer()
            .map(|f| {
                format!(
                    r#"<div class="{footer_class}">{f}</div>"#,
                    footer_class = BloxCss::footer_class()
                )
            })
            .unwrap_or_default();

        let id: String = blox.id_str(config).unwrap_or("".to_string());
        let group_str = config.group_str(blox.env()).unwrap();

        format!(
            r####"<div id="{id}" class="{block_class} {group_str}">{header}<div class="{content_class}">
{content}
</div>{footer}</div>
"####
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::default_test_config;
    use crate::parse::{Blox, BloxOptions};
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    fn check_html(blox: Blox, expected: &str) -> Result<()> {
        let config = default_test_config();
        let html = BloxRender::html(&config, &blox, "", None);

        assert_eq!(html, expected.to_string());
        Ok(())
    }

    #[test]
    fn test_html() -> Result<()> {
        check_html(
            Blox::new(
                "alert",
                BloxOptions {
                    title: None,
                    footer: None,
                    label: None,
                    defer_rendering: false,
                    hide_name: None,
                    hide_header: None,
                    number: None,
                },
            ),
            r#"<div id="" class="blox blox-alert"><div class="blox-header">Alert</div><div class="blox-content">

</div></div>
"#,
        )?;

        check_html(
            Blox::new(
                "exercise",
                BloxOptions {
                    title: None,
                    footer: None,
                    label: None,
                    defer_rendering: false,
                    hide_name: None,
                    hide_header: None,
                    number: Some(10),
                },
            ),
            r#"<div id="" class="blox blox-exercise"><div class="blox-header">Exercise 10</div><div class="blox-content">

</div></div>
"#,
        )?;

        check_html(
            Blox::new(
                "alert",
                BloxOptions {
                    title: None,
                    footer: None,
                    label: Some("warning-22".to_string()),
                    defer_rendering: false,
                    hide_name: None,
                    hide_header: None,
                    number: Some(10),
                },
            ),
            r#"<div id="blox-alert-warning-22" class="blox blox-alert"><div class="blox-header">Alert 10</div><div class="blox-content">

</div></div>
"#,
        )?;

        Ok(())
    }
}
