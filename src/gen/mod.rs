mod cover;
mod transform;

pub mod epub;
pub mod html;

use crate::{
    types::{BoardInPost, Character, Icon, User},
    Post, Reply,
};

use super::Thread;

const STYLE: &str = include_str!("book.css");

#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    pub text_to_speech: bool,
    pub flatten_details: bool,
}

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
    let author_ids: Vec<u64> = authors.iter().map(|user| user.id).collect();

    let BoardInPost {
        id: board_id,
        name: board_name,
    } = board;

    let description = description
        .as_ref()
        .map(|v| fix_content(v, false))
        .map(|v| format!(r##"<div class="description">{v}</div>"##))
        .unwrap_or_default();

    format!(
        r##"

    <div class="title-page">
        <h1 post-id="{id}">{subject}</h1>
        <h2 author-ids="{author_ids:?}">by {author_names}</h2>
        <h3 board-id="{board_id}">in {board_name}</h3>
        <p>[Status: <a href="https://glowfic.com/posts/{id}" rel="noopener noreferrer">{status}</a>]</p>
        <p>[{reply_count} replies]</p>
        {description}
    </div>

    "##
    )
}

pub fn raw_content_page(content_blocks: &[String]) -> String {
    let content: String = content_blocks
        .iter()
        .map(String::as_ref)
        .collect::<Vec<_>>()
        .join("<hr>");

    format!(
        r##"

        <div class="content">
            {content}
        </div>

        "##
    )
}

impl Thread {
    fn content_blocks(&self, options: Options) -> Vec<String> {
        std::iter::once(self.post.content_block(options))
            .chain(
                self.replies
                    .iter()
                    .map(|reply| reply.content_block(options)),
            )
            .collect()
    }
}
impl Post {
    fn content_block(&self, options: Options) -> String {
        content_block(
            None,
            &None,
            &self.character,
            &self.icon,
            &self.content,
            options,
        )
    }
}
impl Reply {
    fn content_block(&self, options: Options) -> String {
        content_block(
            Some(self.id),
            &Some(self.user.clone()),
            &self.character,
            &self.icon,
            &self.content,
            options,
        )
    }
}

fn content_block(
    reply_id: Option<u64>,
    author: &Option<User>,
    character: &Option<Character>,
    icon: &Option<Icon>,
    content: &str,
    options: Options,
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
                .map(|n| transform::encode_html(&n))
                .unwrap_or_default();
            let character_name = transform::encode_html(character_name);

            match author {
                Some(User {
                    id: user_id,
                    username,
                }) => {
                    let username = transform::encode_html(username);
                    let author_line = if options.text_to_speech {
                        format!(r##"{username} <br>as {character_name}"##)
                    } else {
                        format!(r##"{username} <br>as {character_name} <br>{screenname}"##)
                    };

                    format!(
                        r##"
                    <span author-id="{user_id}" author-name="{username}" character-id="{character_id}" character-name="{character_name}" class="icon-caption">
                    {author_line}
                    </span>
                    "##
                    )
                }
                None => {
                    let author_line = if options.text_to_speech {
                        character_name.clone()
                    } else {
                        format!(r##"{character_name} <br>{screenname}"##)
                    };

                    format!(
                        r##"
                    <span character-id="{character_id}" character-name="{character_name}" class="icon-caption">
                    {author_line}
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
            }) => {
                let username = transform::encode_html(username);
                format!(
                    r##"<span author-id="{user_id}" author-name="{username}" class="icon-caption">{username}</span>"##
                )
            }

            None => "".to_string(),
        },
    };
    let image = icon
        .as_ref()
        .and_then(|Icon { id, keyword, url }| {
            let keyword = keyword
                .as_deref()
                .map(transform::encode_html)
                .map(|keyword| format!(r#" alt="{keyword}""#))
                .unwrap_or_default();
            let url = url.as_ref()?;
            let url = transform::encode_html(url);
            Some(format!(
                r##"<img src="{url}"{keyword} icon-id="{id}" class="icon">"##
            ))
        })
        .unwrap_or_default();

    let content = fix_content(content, options.flatten_details);

    let reply_id = reply_id
        .map(|id| format!(r##" reply-id="{id}""##))
        .unwrap_or_default();

    format!(
        r##"

    <div class="content-block"{reply_id}>
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
    let author_ids: Vec<u64> = authors.iter().map(|user| user.id).collect();

    let BoardInPost {
        id: board_id,
        name: board_name,
    } = board;

    format!(
        r##"

    <div class="copyright-page">
        <h3>This was</h3>
        <h1 post-id="{id}">{subject}</h1>
        <h2 author-ids="{author_ids:?}">by {author_names}</h2>
        <h3 board-id="{board_id}">in {board_name}</h3>

        © {author_names}
    </div>

    "##
    )
}

fn author_names(authors: &[User]) -> String {
    let usernames: Vec<_> = authors
        .iter()
        .map(|a| transform::encode_html(&a.username))
        .collect();

    match &*usernames {
        [] => String::new(),
        [username] => username.to_string(),
        [one, two] => format!("{one} &#38; {two}"),
        [leading @ .., last] => {
            let leading = leading.join(", ");
            format!("{leading}, &#38; {last}")
        }
    }
}

fn fix_content(content: &str, flatten_details: bool) -> String {
    let content = transform::repair_and_sanitize(content);
    let content = transform::decode_named_entities(content);

    if flatten_details {
        transform::flatten_details(&content).unwrap()
    } else {
        content
    }
}
