use std::time::Duration;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{Board, Post, Reply};

const GLOWFIC_API_V1: &str = "https://www.glowfic.com/api/v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlowficResponse<T> {
    Value(T),
    Error { errors: Vec<GlowficError> },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct Replies(pub(crate) Vec<Reply>);

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

impl Replies {
    pub fn page_url(id: u64, page: u64) -> String {
        format!("{GLOWFIC_API_V1}/posts/{id}/replies?page={page}")
    }

    async fn get_page(
        id: u64,
        page: u64,
    ) -> Result<Result<Vec<Reply>, Vec<GlowficError>>, reqwest::Error> {
        get_glowfic(&Self::page_url(id, page)).await
    }

    pub async fn get_all(id: u64) -> Result<Result<Vec<Reply>, Vec<GlowficError>>, reqwest::Error> {
        let mut replies = vec![];

        for page in 1.. {
            match Self::get_page(id, page).await? {
                Ok(mut inner_replies) => {
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

pub(crate) async fn get_glowfic<T>(
    url: &str,
) -> Result<Result<T, Vec<GlowficError>>, reqwest::Error>
where
    T: DeserializeOwned,
{
    let response = reqwest::get(url).await?;
    let parsed: GlowficResponse<T> = response.json().await?;

    Ok(parsed.to_result())
}

impl<T> GlowficResponse<T> {
    pub fn to_result(self) -> Result<T, Vec<GlowficError>> {
        match self {
            GlowficResponse::Value(value) => Ok(value),
            GlowficResponse::Error { errors } => Err(errors),
        }
    }
}
