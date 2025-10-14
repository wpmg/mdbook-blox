use crate::config::Config;
use crate::css::BloxCss;
use crate::parse::Blox;

pub struct BloxRender;
impl BloxRender {
    // Returns None if header should be hidden
    fn header(config: &Config, blox: &Blox) -> Option<String> {
        match blox.hide_header() {
            true => None,
            false => blox.title_full(config),
        }
    }

    pub fn html(config: &Config, blox: &Blox) -> String {
        let block_class = BloxCss::block_class();
        let content_class = BloxCss::content_class();

        let header = Self::header(config, blox)
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
            r####"<div id="{id}" class="{block_class} {group_str}">{header}<div class="{content_class}">{content}</div>{footer}</div>"####,
            content = blox.content
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::default_test_config;
    use crate::parse::Blox;
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    fn check_html(blox: Blox, expected: &str) -> Result<()> {
        let config = default_test_config();
        let html = BloxRender::html(&config, &blox);

        assert_eq!(html, expected.to_string());
        Ok(())
    }

    #[test]
    fn test_html() -> Result<()> {
        check_html(
            {
                let blox = Blox::new("alert");
                blox
            },
            r#"<div id="" class="blox blox-alert"><div class="blox-header">Alert</div><div class="blox-content"></div></div>"#,
        )?;

        check_html(
            {
                let mut blox = Blox::new("exercise");
                blox.number = Some("10".to_string());
                blox
            },
            r#"<div id="" class="blox blox-exercise"><div class="blox-header">Exercise 10</div><div class="blox-content"></div></div>"#,
        )?;

        check_html(
            {
                let mut blox = Blox::new("alert");
                blox.number = Some("10".to_string());
                blox.label = Some("warning-22".to_string());
                blox
            },
            r#"<div id="blox-alert-warning-22" class="blox blox-alert"><div class="blox-header">Alert 10</div><div class="blox-content"></div></div>"#,
        )?;

        Ok(())
    }
}
