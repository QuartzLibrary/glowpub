use std::borrow::Cow;

use lol_html::{
    html_content::ContentType, rewrite_str, ElementContentHandlers, RewriteStrSettings,
};

pub fn flatten_details(content: &str) -> String {
    rewrite_str(
        content,
        RewriteStrSettings {
            element_content_handlers: vec![
                (
                    Cow::Owned("details".parse().unwrap()),
                    ElementContentHandlers::default().element(|el| {
                        el.set_tag_name("blockquote").unwrap();
                        Ok(())
                    }),
                ),
                (
                    Cow::Owned("summary".parse().unwrap()),
                    ElementContentHandlers::default().element(|el| {
                        el.set_tag_name("p").unwrap();
                        el.prepend("▼ ", ContentType::Text);
                        Ok(())
                    }),
                ),
            ],
            ..RewriteStrSettings::default()
        },
    )
    .unwrap()
}
