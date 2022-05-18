mod cover;
pub mod epub;
pub mod html;
mod named_entities;

use std::borrow::Cow;

use crate::{
    types::{BoardInPost, Character, Icon, User},
    Post, Reply,
};

use super::Thread;

const STYLE: &str = include_str!("book.css");

fn raw_title_page(post: &Post, reply_count: usize) -> String {
    let Post {
        authors,
        board,
        description,
        id,
        status,
        subject,
        ..
    } = post;

    let author_names = author_names(authors);
    let author_ids: String = format!(
        "{:?}",
        authors.iter().map(|user| user.id).collect::<Vec<_>>()
    );

    let BoardInPost {
        id: board_id,
        name: board_name,
    } = board;

    let description = description
        .as_ref()
        .map(|v| format!(r##"<div class="description">{v}</div>"##))
        .unwrap_or_default();

    let description = fix_content(&description);

    format!(
        r##"

    <div class="title-page">
        <h1 class="title" post-id="{id}">{subject}</h1>
        <h2 class="authors" glowfic-ids="{author_ids}">by {author_names}</h2>
        <h3 class="board" board-id="{board_id}">in {board_name}</h3>
        <p class="status">[Status: <a href="https://glowfic.com/posts/{id}">{status}</a>]</p>
        <p class="reply-count">[{reply_count} replies]</p>
        {description}
    </div>

    "##
    )
}

pub fn raw_content_page(content_blocks: &[String]) -> String {
    let content: String = content_blocks
        .iter()
        .map(String::as_ref)
        .intersperse("<hr/>")
        .collect();

    format!(
        r##"

        <div class="content">
            {content}
        </div>

        "##
    )
}

impl Thread {
    fn content_blocks(&self) -> Vec<String> {
        std::iter::once(self.post.content_block())
            .chain(self.replies.iter().map(|reply| reply.content_block()))
            .collect()
    }
}
impl Post {
    fn content_block(&self) -> String {
        content_block(&None, &self.character, &self.icon, &self.content)
    }
}
impl Reply {
    fn content_block(&self) -> String {
        content_block(
            &Some(self.user.clone()),
            &self.character,
            &self.icon,
            &self.content,
        )
    }
}

fn content_block(
    author: &Option<User>,
    character: &Option<Character>,
    icon: &Option<Icon>,
    content: &str,
) -> String {
    let caption = match character {
        Some(Character {
            id: character_id,
            name: character_name,
            screenname,
        }) => {
            let screenname = screenname
                .as_ref()
                .map(|n| format!("({n})"))
                .unwrap_or_default();

            match author {
                Some(User {
                    id: user_id,
                    username,
                }) => {
                    let character_name = character_name.replace('"', "&quot;");
                    format!(
                        r##"
                    <span author-id="{user_id}" author-name="{username}" character-id="{character_id}" character-name="{character_name}" class="icon-caption">
                    {username} <br/>as {character_name} <br/>{screenname}
                    </span>
                    "##
                    )
                }
                None => {
                    let character_name = character_name.replace('"', "&quot;");
                    format!(
                        r##"
                    <span character-id="{character_id}" character-name="{character_name}" class="icon-caption">
                    {screenname}
                    </span>
                    "##
                    )
                }
            }
        }
        None => match author {
            Some(User {
                id: user_id,
                username,
            }) => format!(
                r##"<span author-id="{user_id}" author-name="{username}" class="icon-caption">{username}</span>"##
            ),
            None => "".to_string(),
        },
    };
    let image = icon
        .as_ref()
        .map(|Icon { id, keyword, url }| {
            format!(r##"<img src="{url}" alt="{keyword}" glowfic-id="{id}" class="icon"/>"##)
        })
        .unwrap_or_default();

    let content = fix_content(content);

    format!(
        r##"

    <div class="content-block">
        <div class="character">
            {image}
            {caption}
        </div>
        {content}
    </div>

    "##
    )
}

fn raw_copyright_page(post: &Post) -> String {
    let Post {
        authors,
        board,
        id,
        subject,
        ..
    } = post;

    let author_names = author_names(authors);
    let author_ids: String = format!(
        "{:?}",
        authors.iter().map(|user| user.id).collect::<Vec<_>>()
    );

    let BoardInPost {
        id: board_id,
        name: board_name,
    } = board;

    format!(
        r##"

    <div class="copyright-page">
        <h3>This was</h3>
        <h1 class="title" post-id="{id}">{subject}</h1>
        <h2 class="authors" glowfic-ids="{author_ids}">by {author_names}</h2>
        <h3 class="board" board-id="{board_id}">in {board_name}</h3>

        © {author_names}
    </div>

    "##
    )
}

fn author_names(authors: &[User]) -> String {
    match authors {
        [User { username, .. }] => username.to_string(),
        [User { username: one, .. }, User { username: two, .. }] => format!("{one} &#38; {two}"),
        _ => authors[..authors.len() - 1]
            .iter()
            .map(|user| user.username.as_str())
            .intersperse(", ")
            .chain(std::iter::once(", and "))
            .chain(std::iter::once(
                authors[authors.len() - 1].username.as_str(),
            ))
            .collect(),
    }
}

/// Fixes internal relative links by adding the resto of the glowfic url.
fn fix_content(content: &str) -> String {
    let content = repair_and_sanitize(content);
    let content = named_entities::decode_named_entities(content);

    // ammonia (or html5ever), seems to interpret '<tag />' as `<tag>` instead of `<tag/>`,
    // which breaks some things. This is not great because `<br></br>` is also valid.
    // TODO: do a regex replacement beforehand that is resistant to varying whitespace.
    content.replace("<br>", "<br/>").replace("<hr>", "<hr/>")
}

fn repair_and_sanitize(content: &str) -> String {
    let builder = {
        let mut builder = ammonia::Builder::default();

        for (tag, classes) in ALLOWED_CLASSES {
            builder.add_allowed_classes(tag, classes);
        }

        builder.url_relative(ammonia::UrlRelative::RewriteWithBase(
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

const ALLOW_LISTED_STYLES: [&str; 5] = [
    "width: auto;",
    "border: 0;",
    "text-decoration-line: line-through;",
    "text-decoration: line-through;",
    "text-decoration: underline;",
];
