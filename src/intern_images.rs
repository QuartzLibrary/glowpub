use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::Cursor,
};

use image::imageops::FilterType;
use mime::Mime;

use crate::{
    cached::download_cached_image,
    types::{Continuity, Icon, Thread},
    utils::{mime_to_image_extension, url_hash},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternedImage {
    /// [None] means it was not an icon but an inline url.
    pub id: Option<usize>,
    pub original_url: String,
    pub mime: Mime,
    pub data: Vec<u8>,
}
impl InternedImage {
    pub fn is_icon(&self) -> bool {
        self.id.is_some()
    }
    pub fn name(&self) -> String {
        let Self { id, mime, .. } = self;

        let extension = mime_to_image_extension(mime).expect("interned image should be image");

        // Note: epub file names should start with a letter for maximum compatibility.
        match id {
            Some(id) => format!("glowfic_{id}.{extension}"),
            None => {
                let url_hash = url_hash(&self.original_url);
                format!("hash_{url_hash}.{extension}")
            }
        }
    }
    /// Converts some image formats into more widely supported ones for epub compatibility.
    pub fn into_common_format(self) -> Self {
        match (self.mime.type_(), self.mime.subtype()) {
            (mime::IMAGE, mime::BMP)
            | (mime::IMAGE, mime::GIF)
            | (mime::IMAGE, mime::JPEG)
            | (mime::IMAGE, mime::PNG)
            | (mime::IMAGE, mime::SVG) => self,
            (mime::IMAGE, subtype) if subtype.as_str() == "webp" => self.into_png(),
            _ => unreachable!(),
        }
    }
    pub fn try_into_jpeg(self) -> Self {
        match (self.mime.type_(), self.mime.subtype()) {
            (mime::IMAGE, mime::BMP)
            | (mime::IMAGE, mime::GIF)
            | (mime::IMAGE, mime::JPEG)
            | (mime::IMAGE, mime::PNG) => self.into_jpeg(),
            (mime::IMAGE, subtype) if subtype.as_str() == "webp" => self.into_jpeg(),

            (mime::IMAGE, mime::SVG) => self,
            _ => unreachable!(),
        }
    }
    pub fn resize_down(self, width: u32) -> Result<Self, Box<dyn Error>> {
        let img = self.to_dynamic_image()?;

        if img.width() < width {
            return Ok(self);
        }

        let Ok(format) = self.image_format() else {
            // Avoid resizing unsupported formats.
            return Ok(self);
        };

        let img = img.resize(width, img.height(), FilterType::Lanczos3);

        let mut data = Vec::with_capacity(self.data.len());
        img.write_to(&mut Cursor::new(&mut data), format)?;

        Ok(Self {
            id: self.id,
            original_url: self.original_url,
            mime: self.mime,
            data,
        })
    }
}
impl InternedImage {
    fn to_dynamic_image(&self) -> Result<image::DynamicImage, Box<dyn Error>> {
        Ok(image::load(Cursor::new(&self.data), self.image_format()?)?)
    }
    fn image_format(&self) -> Result<image::ImageFormat, Box<dyn Error>> {
        Ok(match (self.mime.type_(), self.mime.subtype()) {
            (mime::IMAGE, mime::BMP) => image::ImageFormat::Bmp,
            (mime::IMAGE, mime::GIF) => image::ImageFormat::Gif,
            (mime::IMAGE, mime::JPEG) => image::ImageFormat::Jpeg,
            (mime::IMAGE, mime::PNG) => image::ImageFormat::Png,
            (mime::IMAGE, mime::SVG) => Err("svg not supported")?,
            (mime::IMAGE, subtype) if subtype.as_str() == "webp" => image::ImageFormat::WebP,
            _ => unreachable!(),
        })
    }
    fn into_png(self) -> Self {
        let id = self.id;
        let original_url = self.original_url.clone();

        let mut data = Vec::with_capacity(self.data.len());

        self.to_dynamic_image()
            .unwrap()
            .write_to(&mut Cursor::new(&mut data), image::ImageFormat::Png)
            .unwrap();

        Self {
            id,
            original_url,
            mime: mime::IMAGE_PNG,
            data,
        }
    }
    fn into_jpeg(self) -> Self {
        let id = self.id;
        let original_url = self.original_url.clone();

        let mut data = Vec::with_capacity(self.data.len());

        self.to_dynamic_image()
            .unwrap()
            .into_rgb8()
            .write_to(&mut Cursor::new(&mut data), image::ImageFormat::Jpeg)
            .unwrap();

        Self {
            id,
            original_url,
            mime: mime::IMAGE_JPEG,
            data,
        }
    }
}

impl Continuity {
    pub async fn images_to_intern(&self) -> Result<HashMap<String, InternedImage>, Box<dyn Error>> {
        let mut interned_images: HashMap<String, InternedImage> = HashMap::new();
        let mut skip: HashSet<String> = HashSet::default();

        for thread in &self.threads {
            thread
                .icons_to_intern(&mut interned_images, &mut skip)
                .await?;

            thread
                .image_tags_to_intern(&mut interned_images, &mut skip)
                .await?;
        }

        Ok(interned_images)
    }
}

impl Thread {
    pub async fn images_to_intern(&self) -> Result<HashMap<String, InternedImage>, Box<dyn Error>> {
        let mut interned_images: HashMap<String, InternedImage> = HashMap::new();
        let mut skip: HashSet<String> = HashSet::default();

        self.icons_to_intern(&mut interned_images, &mut skip)
            .await?;

        self.image_tags_to_intern(&mut interned_images, &mut skip)
            .await?;

        Ok(interned_images)
    }

    async fn icons_to_intern(
        &self,
        interned_images: &mut HashMap<String, InternedImage>,
        skip: &mut HashSet<String>,
    ) -> Result<(), Box<dyn Error>> {
        for icon in self.icons() {
            let Some(url) = icon.url.clone() else {
                continue;
            };

            if skip.contains(&url) || interned_images.contains_key(&url) {
                continue;
            }

            match icon.intern().await {
                Ok(interned) => interned_images.insert(url, interned.into_common_format()),
                Err(e) => {
                    let id = icon.id;
                    log::info!(
                        "Was unable to retrieve icon {id}, the original url will be inlined (url: {url}).\n{e:?}"
                    );
                    skip.insert(url);
                    continue;
                }
            };
        }

        Ok(())
    }

    async fn image_tags_to_intern(
        &self,
        interned_images: &mut HashMap<String, InternedImage>,
        skip: &mut HashSet<String>,
    ) -> Result<(), Box<dyn Error>> {
        for url in self.image_urls() {
            if skip.contains(&url) || interned_images.contains_key(&url) {
                continue;
            }

            match download_cached_image(&url, false).await {
                Ok((mime, data)) => {
                    interned_images.insert(
                        url.clone(),
                        InternedImage {
                            id: None,
                            original_url: url,
                            mime,
                            data,
                        }
                        .into_common_format(),
                    );
                }
                Err(e) => {
                    log::info!(
                        "Was unable to retrieve image, the original url will be inlined (url: {url}).\n{e:?}"
                    );
                    skip.insert(url);
                    continue;
                }
            }
        }

        Ok(())
    }
}

impl Icon {
    async fn intern(&self) -> Result<InternedImage, Box<dyn Error>> {
        let (mime, data) = self.download_cached(false).await?;

        Ok(InternedImage {
            id: Some(self.id.try_into().unwrap()),
            original_url: self.url.clone().unwrap(),
            mime,
            data,
        })
    }
}
