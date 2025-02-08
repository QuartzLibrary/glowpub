use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)] // Not serialized
pub struct Continuity {
    pub board: Board,
    pub threads: Vec<Thread>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Board {
    pub id: u64,
    pub name: String,
    pub board_sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub id: u64,
    pub name: String,
    pub order: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)] // Not serialized
pub struct Thread {
    pub post: Post,
    pub replies: Vec<Reply>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Post {
    pub id: u64,
    pub authors: Vec<User>,
    pub board: BoardInPost,
    pub character: Option<Character>,
    pub content: String,
    #[serde(with = "crate::rfc3339")]
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub icon: Option<Icon>,
    pub num_replies: u64,
    pub section: Option<Section>,
    pub section_order: u64,
    pub status: String,
    pub subject: String,
    #[serde(with = "crate::rfc3339")]
    pub tagged_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub id: u64,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Token {
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardInPost {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Character {
    pub id: u64,
    pub name: String,
    pub screenname: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Icon {
    pub id: u64,
    pub keyword: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub id: u64,
    pub character: Option<Character>,
    /// The name being used for this character in this reply.
    /// This will be an alias if one was used, or else the same as `self.character.name`.
    pub character_name: Option<String>,
    pub content: String,
    #[serde(with = "crate::rfc3339")]
    pub created_at: DateTime<Utc>,
    pub icon: Option<Icon>,
    #[serde(with = "crate::rfc3339")]
    pub updated_at: DateTime<Utc>,
    pub user: User,
}

mod helpers {
    use std::{
        collections::{BTreeSet, HashSet},
        iter,
    };

    use crate::gen::transform;

    use super::*;

    // TODO: can we rely on there always being at least one thread?
    impl Continuity {
        pub fn created_at(&self) -> Option<DateTime<Utc>> {
            self.threads.iter().map(|t| t.post.created_at).min()
        }
        pub fn tagged_at(&self) -> Option<DateTime<Utc>> {
            self.threads.iter().map(|t| t.post.tagged_at).max()
        }
        pub fn authors(&self) -> Vec<User> {
            self.threads
                .iter()
                .flat_map(|t| t.post.authors.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        }
        pub fn sections(&self) -> (Vec<(Section, Vec<&Thread>)>, Vec<&Thread>) {
            let mut sections = self.board.board_sections.clone();
            sections.sort_by_key(|s| s.order);

            let sections: Vec<(Section, Vec<&Thread>)> = sections
                .into_iter()
                .map(|s| {
                    let mut threads: Vec<&Thread> = self
                        .threads
                        .iter()
                        .filter(|t| t.post.section.as_ref().map(|s| s.id) == Some(s.id))
                        .collect();
                    threads.sort_by_key(|t| t.post.section_order);
                    (s, threads)
                })
                .collect();

            let mut sectionless_threads: Vec<&Thread> = self
                .threads
                .iter()
                .filter(|t| t.post.section.is_none())
                .collect();
            sectionless_threads.sort_by_key(|t| t.post.section_order);

            (sections, sectionless_threads)
        }
    }
    impl Thread {
        pub fn image_urls(&self) -> HashSet<String> {
            let contents = iter::once(&self.post.content)
                .chain(&self.post.description)
                .chain(self.replies.iter().map(|r| &r.content));

            let mut urls = HashSet::new();
            for c in contents {
                transform::edit_image_urls(c, |url| {
                    urls.insert(url.clone());
                    url
                });
            }

            urls
        }
    }
}
