use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

use glowfic_to_epub::{
    api::{GlowficError, GlowficResponse, Replies},
    Board, Post,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const ALL_POSTS: &str = "./tests/fixtures/api-posts.json";
const OK_POSTS: &str = "./tests/fixtures/api-posts-success.json";
const ERR_POSTS: &str = "./tests/fixtures/api-posts-error.json";

const ALL_REPLIES: &str = "./tests/fixtures/api-replies.json";
const OK_REPLIES: &str = "./tests/fixtures/api-replies-success.json";
const ERR_REPLIES: &str = "./tests/fixtures/api-replies-error.json";

const ALL_BOARDS: &str = "./tests/fixtures/api-boards.json";
const OK_BOARDS: &str = "./tests/fixtures/api-boards-success.json";
const ERR_BOARDS: &str = "./tests/fixtures/api-boards-error.json";

type PostResponse = GlowficResponse<Post>;
type RepliesResponse = GlowficResponse<Replies>;
type BoardResponse = GlowficResponse<Board>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Error {
    errors: Vec<GlowficError>,
}

#[tokio::test]
async fn deserialisation() -> Result<()> {
    let _posts: Vec<PostResponse> = serde_json::from_str(&read_to_string(ALL_POSTS)?)?;
    let _posts: Vec<Post> = serde_json::from_str(&read_to_string(OK_POSTS)?)?;
    let _posts: Vec<Error> = serde_json::from_str(&read_to_string(ERR_POSTS)?)?;

    let _replies: Vec<RepliesResponse> = serde_json::from_str(&read_to_string(ALL_REPLIES)?)?;
    let _replies: Vec<Replies> = serde_json::from_str(&read_to_string(OK_REPLIES)?)?;
    let _replies: Vec<Error> = serde_json::from_str(&read_to_string(ERR_REPLIES)?)?;

    let _boards: Vec<BoardResponse> = serde_json::from_str(&read_to_string(ALL_BOARDS)?)?;
    let _boards: Vec<Board> = serde_json::from_str(&read_to_string(OK_BOARDS)?)?;
    let _boards: Vec<Error> = serde_json::from_str(&read_to_string(ERR_BOARDS)?)?;

    Ok(())
}
