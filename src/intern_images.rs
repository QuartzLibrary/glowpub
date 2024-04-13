use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::Cursor,
};

use mime::Mime;

use crate::{
    types::{Continuity, Icon, Thread},
    utils::mime_to_image_extension,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternedImage {
    pub id: usize,
    pub original_url: String,
    pub mime: Mime,
    pub data: Vec<u8>,
}
impl InternedImage {
    pub fn name(&self) -> String {
        let Self { id, mime, .. } = self;

        let extension = mime_to_image_extension(mime).expect("interned image should be image");

        // `glowfic_` is to ensure epub ids start with a letter.
        format!("glowfic_{id}.{extension}")
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
}

impl Continuity {
    pub async fn images_to_intern(&self) -> Result<HashMap<String, InternedImage>, Box<dyn Error>> {
        let mut interned_images: HashMap<String, InternedImage> = HashMap::new();
        let mut skip: HashSet<u64> = HashSet::default();

        for thread in &self.threads {
            thread
                .images_to_intern_inner(&mut interned_images, &mut skip)
                .await?;
        }

        Ok(interned_images)
    }
}

impl Thread {
    pub async fn images_to_intern(&self) -> Result<HashMap<String, InternedImage>, Box<dyn Error>> {
        let mut interned_images: HashMap<String, InternedImage> = HashMap::new();
        let mut skip: HashSet<u64> = HashSet::default();

        self.images_to_intern_inner(&mut interned_images, &mut skip)
            .await?;

        Ok(interned_images)
    }

    async fn images_to_intern_inner(
        &self,
        interned_images: &mut HashMap<String, InternedImage>,
        skip: &mut HashSet<u64>,
    ) -> Result<(), Box<dyn Error>> {
        for icon in self.icons() {
            let Some(url) = icon.url.clone() else {
                continue;
            };

            if skip.contains(&icon.id) || interned_images.contains_key(&url) {
                continue;
            }

            match icon.intern().await {
                Ok(interned) => interned_images.insert(url, interned.into_common_format()),
                Err(e) => {
                    let id = icon.id;
                    log::info!(
                        "Was unable to retrieve icon {id}, the original url will be inlined."
                    );
                    log::info!("{e:?}");
                    skip.insert(icon.id);
                    continue;
                }
            };
        }

        Ok(())
    }
}

impl Icon {
    async fn intern(&self) -> Result<InternedImage, Box<dyn Error>> {
        let (mime, data) = self.retrieve_cached(false).await?;
        Ok(InternedImage {
            id: self.id.try_into().unwrap(),
            original_url: self.url.clone().unwrap(),
            mime,
            data,
        })
    }
}
