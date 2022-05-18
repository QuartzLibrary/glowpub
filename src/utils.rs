use mime::Mime;

use crate::types::{Icon, Thread};

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
        _ => None,
    }
    .map(str::to_string)
}

pub fn extension_to_image_mime(extension: &str) -> Option<Mime> {
    match extension {
        "bmp" => Some(mime::IMAGE_BMP),
        "gif" => Some(mime::IMAGE_GIF),
        "jpg" | "jpeg" => Some(mime::IMAGE_JPEG),
        "png" => Some(mime::IMAGE_PNG),
        "svg" => Some(mime::IMAGE_SVG),
        _ => None,
    }
}
