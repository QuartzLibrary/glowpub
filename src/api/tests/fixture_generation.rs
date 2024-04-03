use std::{ops::Range, time::Duration};

use rand::{distributions::Uniform, Rng};
use serde_json::Value;

use crate::{Board, Post};

use super::super::Replies;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore]
async fn gen_boards_file() -> Result<()> {
    let urls: Vec<String> = iter_rng(0..4_000).take(100).map(Board::url).collect();

    let responses: Vec<Value> = to_responses(&urls).await?;

    save_to_file(&responses, "boards")?;

    Ok(())
}

#[tokio::test]
#[ignore]
async fn gen_posts_file() -> Result<()> {
    let urls: Vec<String> = iter_rng(0..4_000).take(100).map(Post::url).collect();

    let responses: Vec<Value> = to_responses(&urls).await?;

    save_to_file(&responses, "posts")?;

    Ok(())
}

#[tokio::test]
#[ignore]
async fn gen_replies_file() -> Result<()> {
    let urls: Vec<String> = iter_rng(0..4_000)
        .take(100)
        .flat_map(|id| (0..10).map(|page| (id, page)).collect::<Vec<_>>())
        .map(|(id, page)| Replies::page_url(id, page))
        .collect();

    let responses: Vec<Value> = to_responses(&urls).await?;

    save_to_file(&responses, "replies")?;

    Ok(())
}

pub async fn to_responses(urls: &[String]) -> Result<Vec<Value>> {
    let mut responses: Vec<Value> = vec![];

    for url in urls {
        responses.push(reqwest::get(url).await?.json().await?);

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(responses)
}

pub fn save_to_file(responses: &[Value], name: &str) -> Result<()> {
    std::fs::write(
        format!("./src/api/tests/fixtures/api-{name}.json"),
        serde_json::to_string(responses)?,
    )?;
    std::fs::write(
        format!("./src/api/tests/fixtures/api-{name}-success.json"),
        serde_json::to_string(&only_successes(responses))?,
    )?;
    std::fs::write(
        format!("./src/api/tests/fixtures/api-{name}-error.json"),
        serde_json::to_string(&only_errors(responses))?,
    )?;
    Ok(())
}

pub fn only_successes(responses: &[Value]) -> Vec<Value> {
    responses
        .iter()
        .filter(|response| !is_error(response))
        .cloned()
        .collect()
}

pub fn only_errors(responses: &[Value]) -> Vec<Value> {
    responses
        .iter()
        .filter(|response| is_error(response))
        .cloned()
        .collect()
}

pub fn is_error(response: &Value) -> bool {
    match response {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            panic!("{response:?}")
        }
        Value::Array(array) => {
            assert!(array
                .iter()
                .all(|v| !v.as_object().unwrap().contains_key("errors")));

            false
        }
        Value::Object(object) => object.contains_key("errors"),
    }
}

fn iter_rng(range: Range<u64>) -> impl Iterator<Item = u64> {
    rand::thread_rng().sample_iter(Uniform::from(range))
}
