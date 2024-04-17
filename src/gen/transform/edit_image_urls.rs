use std::borrow::Cow;

use lol_html::{rewrite_str, ElementContentHandlers, RewriteStrSettings};

pub fn edit_image_urls(content: &str, mut f: impl FnMut(String) -> String) -> String {
    rewrite_str(
        content,
        RewriteStrSettings {
            element_content_handlers: vec![(
                Cow::Owned("img".parse().unwrap()),
                ElementContentHandlers::default().element(|el| {
                    if let Some(url) = el.get_attribute("src") {
                        let new_url = f(url);
                        el.set_attribute("src", &new_url).unwrap();
                    }
                    Ok(())
                }),
            )],
            ..RewriteStrSettings::default()
        },
    )
    .unwrap()
}
