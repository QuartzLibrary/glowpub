#[cfg(test)]
mod tests;

use std::{future::Future, time::Duration};

use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    types::{BoardInPost, Section, User},
    Board, Post, Reply,
};

const GLOWFIC_API_V1: &str = "https://www.glowfic.com/api/v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
enum GlowficResponse<T> {
    Value(T),
    Error { errors: Vec<GlowficError> },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GlowficError {
    message: String,
}

impl Board {
    pub fn url(id: u64) -> String {
        format!("{GLOWFIC_API_V1}/boards/{id}")
    }

    pub async fn get(id: u64) -> Result<Result<Self, Vec<GlowficError>>, reqwest::Error> {
        get_glowfic(&Self::url(id)).await
    }
}

impl Post {
    pub fn url(id: u64) -> String {
        format!("{GLOWFIC_API_V1}/posts/{id}")
    }

    pub async fn get(id: u64) -> Result<Result<Self, Vec<GlowficError>>, reqwest::Error> {
        get_glowfic(&Self::url(id)).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct Replies(pub(crate) Vec<Reply>);
impl Replies {
    pub fn page_url(id: u64, page: u64) -> String {
        format!("{GLOWFIC_API_V1}/posts/{id}/replies?page={page}")
    }

    async fn get_page(
        id: u64,
        page: u64,
    ) -> Result<Result<Self, Vec<GlowficError>>, reqwest::Error> {
        get_glowfic(&Self::page_url(id, page)).await
    }

    pub async fn get_all(id: u64) -> Result<Result<Vec<Reply>, Vec<GlowficError>>, reqwest::Error> {
        let mut replies = vec![];

        for page in 1.. {
            match Self::get_page(id, page).await? {
                Ok(Self(mut inner_replies)) => {
                    if inner_replies.is_empty() {
                        break;
                    }
                    replies.append(&mut inner_replies);
                }
                Err(errors) => return Ok(Err(errors)),
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(Ok(replies))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardPosts {
    results: Vec<PostInBoard>,
}
/// A subset of [Post].
///
/// Here because it's what this api call uses, but we should normalise it to [Post] everywhere
/// for simplicity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostInBoard {
    pub id: u64,
    pub authors: Vec<User>,
    pub board: BoardInPost,
    #[serde(with = "crate::rfc3339")]
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub num_replies: u64,
    pub section: Option<Section>,
    pub section_order: u64,
    pub status: String,
    pub subject: String,
    #[serde(with = "crate::rfc3339")]
    pub tagged_at: DateTime<Utc>,
}
impl BoardPosts {
    pub fn page_url(id: u64, page: u64) -> String {
        format!("{GLOWFIC_API_V1}/boards/{id}/posts?page={page}")
    }

    async fn get_page(
        id: u64,
        page: u64,
    ) -> Result<Result<Self, Vec<GlowficError>>, reqwest::Error> {
        get_glowfic::<Self>(&Self::page_url(id, page)).await
    }

    pub async fn get_all(
        id: u64,
    ) -> Result<Result<Vec<PostInBoard>, Vec<GlowficError>>, reqwest::Error> {
        let mut posts = vec![];

        for page in 1.. {
            match Self::get_page(id, page).await? {
                Ok(Self { mut results }) => {
                    if results.is_empty() {
                        break;
                    }
                    posts.append(&mut results);
                }
                Err(errors) => return Ok(Err(errors)),
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(Ok(posts))
    }
}

pub(crate) async fn get_glowfic<T>(
    url: &str,
) -> Result<Result<T, Vec<GlowficError>>, reqwest::Error>
where
    T: DeserializeOwned,
{
    let response = retry(5, || reqwest::get(url)).await?;
    let parsed: GlowficResponse<T> = response.json().await?;

    Ok(parsed.into_result())
}

impl<T> GlowficResponse<T> {
    fn into_result(self) -> Result<T, Vec<GlowficError>> {
        match self {
            GlowficResponse::Value(value) => Ok(value),
            GlowficResponse::Error { errors } => Err(errors),
        }
    }
}

/// Executes the closure and its returned [Future].
///
/// If it fails it'll retry up to the provided number of times, for a total of retries+1 attempts.
///
/// Uses an exponential backoff of (1, 10, 100, ...) milliseconds.
pub async fn retry<T, E, Fut: Future<Output = Result<T, E>>>(
    retries: u64,
    mut f: impl FnMut() -> Fut,
) -> Result<T, E> {
    for i in 0..(retries + 1) {
        match f().await {
            Ok(ok) => return Ok(ok),
            Err(e) if i == retries => return Err(e),
            Err(_) => {}
        }
        tokio::time::sleep(Duration::from_millis(10 ^ i)).await;
    }
    unreachable!()
}
