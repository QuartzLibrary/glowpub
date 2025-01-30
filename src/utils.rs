use std::{io::Cursor, str::FromStr, sync::OnceLock};

use image::io::Reader;
use mime::Mime;
use sha2::{Digest, Sha256};

use crate::types::{Icon, Thread};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

impl Thread {
    pub fn icons(&self) -> impl Iterator<Item = &Icon> {
        std::iter::once(self.post.icon.as_ref())
            .flatten()
            .chain(self.replies.iter().flat_map(|r| r.icon.as_ref()))
    }
}

pub fn mime_to_image_extension(mime: &Mime) -> Option<String> {
    match (mime.type_(), mime.subtype()) {
        (mime::IMAGE, mime::BMP) => Some("bmp"),
        (mime::IMAGE, mime::GIF) => Some("gif"),
        (mime::IMAGE, mime::JPEG) => Some("jpeg"),
        (mime::IMAGE, mime::PNG) => Some("png"),
        (mime::IMAGE, mime::SVG) => Some("svg"),
        (mime::IMAGE, subtype) if subtype.as_str() == "webp" => Some("webp"),
        _ => None,
    }
    .map(str::to_string)
}

pub fn guess_image_mime(data: &[u8]) -> Option<Mime> {
    let mime = Reader::new(Cursor::new(data))
        .with_guessed_format()
        .expect("reader shouldn't fail, it is backed by a slice")
        .format()?
        .to_mime_type()
        .parse::<Mime>()
        .expect("`image` crate should return a valid mime");
    Some(mime)
}

pub fn extension_to_image_mime(extension: &str) -> Option<Mime> {
    match extension {
        "bmp" => Some(mime::IMAGE_BMP),
        "gif" => Some(mime::IMAGE_GIF),
        "jpg" | "jpeg" => Some(mime::IMAGE_JPEG),
        "png" => Some(mime::IMAGE_PNG),
        "svg" => Some(mime::IMAGE_SVG),
        "webp" => Some(Mime::from_str("image/webp").unwrap()),
        _ => None,
    }
}

pub fn url_hash(url: &str) -> String {
    let hash: [u8; 32] = Sha256::digest(url).into();
    let hash: [u8; 16] = hash[..16].try_into().unwrap();
    let hash = u128::from_be_bytes(hash);
    format!("{hash:x}")
}

pub fn http_client() -> reqwest::Client {
    // TODO: use global `std::sync::LazyLock` once stable.
    pub static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    HTTP_CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .expect("failed to build http client.")
        })
        .clone()
}

pub trait AnyMap: Sized {
    fn any_map<O>(self, f: impl FnOnce(Self) -> O) -> O;
}
impl<T> AnyMap for T {
    fn any_map<O>(self, f: impl FnOnce(Self) -> O) -> O {
        f(self)
    }
}
