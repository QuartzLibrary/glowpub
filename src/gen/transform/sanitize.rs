use std::borrow::Cow;

use ammonia::{Builder, UrlRelative};

pub fn repair_and_sanitize(content: &str) -> String {
    let builder = {
        let mut builder = Builder::default();

        for (tag, classes) in ALLOWED_CLASSES {
            builder.add_allowed_classes(tag, classes);
        }

        builder.url_relative(UrlRelative::RewriteWithBase(
            "https://glowfic.com/".try_into().unwrap(),
        ));

        builder.add_generic_attributes(["style"]);

        builder.attribute_filter(|_element, attribute, value| {
            if attribute == "style" && !ALLOW_LISTED_STYLES.contains(&value) {
                println!("Style attribute with value \"{value}\" found, removing it for safety.");
                None
            } else {
                Some(Cow::Borrowed(value))
            }
        });

        builder
    };
    let document = builder.clean(content);
    document.to_string()
}

const ALLOWED_CLASSES: [(&str, &[&str]); 7] = [
    (
        "div",
        &[
            "copyright-page",
            "title-page",
            "description",
            "content",
            "character",
        ],
    ),
    ("h1", &["title"]),
    ("h2", &["authors"]),
    ("h3", &["board"]),
    ("img", &["icon"]),
    ("span", &["icon-caption"]),
    ("p", &["status", "reply-count"]),
];

const ALLOW_LISTED_STYLES: [&str; 12] = [
    "width: auto;",
    "border: 0;",
    "text-decoration-line: line-through;",
    "text-decoration: line-through;",
    "text-decoration: underline;",
    "max-width: 20em;",
    "max-width: 30em;",
    "max-width:30em",
    "border: none;",
    "border:none;",
    "line-height: 1.38; margin-top: 0pt; margin-bottom: 0pt;",
    "font-size: 11pt; font-family: Arial; color: #000000; background-color: transparent; font-weight: 400; font-style: normal; font-variant: normal; text-decoration: none; vertical-align: baseline; white-space: pre-wrap;",
];
