use std::str::FromStr;

use mime::Mime;
use reqwest::header::CONTENT_TYPE;

pub async fn retrieve_icon(url: &str) -> Result<(Mime, Vec<u8>), reqwest::Error> {
    let response = reqwest::get(url).await?;

    let content_type = response.headers().get(CONTENT_TYPE).unwrap();
    let mime = Mime::from_str(content_type.to_str().unwrap()).unwrap();

    let data = response.bytes().await?;

    Ok((mime, data.to_vec()))
}
