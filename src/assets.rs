use std::str::FromStr;

use mime::Mime;
use reqwest::header::CONTENT_TYPE;

use crate::types::Icon;

impl Icon {
    pub async fn retrieve(&self) -> Result<(Mime, Vec<u8>), reqwest::Error> {
        let response = reqwest::get(&self.url).await?;

        let content_type = response.headers().get(CONTENT_TYPE).unwrap();
        let mime = Mime::from_str(content_type.to_str().unwrap()).unwrap();

        let data = response.bytes().await?;

        Ok((mime, data.to_vec()))
    }
}
