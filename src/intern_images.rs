use std::{
    collections::{BTreeMap, BTreeSet},
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

        let mut png_data = Vec::with_capacity(self.data.len());

        self.to_dynamic_image()
            .unwrap()
            .write_to(&mut Cursor::new(&mut png_data), image::ImageFormat::Png)
            .unwrap();

        Self {
            id,
            original_url,
            mime: mime::IMAGE_PNG,
            data: png_data,
        }
    }
}

impl Continuity {
    pub async fn intern_images(
        &mut self,
    ) -> Result<BTreeMap<String, InternedImage>, Box<dyn Error>> {
        let mut interned_images: BTreeMap<String, InternedImage> = BTreeMap::new();
        let mut skip: BTreeSet<u64> = BTreeSet::default();

        for thread in &mut self.threads {
            thread
                .intern_images_inner(&mut interned_images, &mut skip)
                .await?;
        }

        Ok(interned_images)
    }
}

impl Thread {
    pub async fn intern_images(
        &mut self,
    ) -> Result<BTreeMap<String, InternedImage>, Box<dyn Error>> {
        let mut interned_images: BTreeMap<String, InternedImage> = BTreeMap::new();
        let mut skip: BTreeSet<u64> = BTreeSet::default();

        self.intern_images_inner(&mut interned_images, &mut skip)
            .await?;

        Ok(interned_images)
    }

    async fn intern_images_inner(
        &mut self,
        interned_images: &mut BTreeMap<String, InternedImage>,
        skip: &mut BTreeSet<u64>,
    ) -> Result<(), Box<dyn Error>> {
        for icon in self.icons_mut() {
            if skip.contains(&icon.id) {
                continue;
            }

            let Some(url) = icon.url.clone() else {
                continue;
            };

            if let Some(interned) = interned_images.get(&url) {
                icon.url = Some(interned.name());
                continue;
            }

            let interned = match icon.intern().await {
                Ok(interned) => interned.into_common_format(),
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

            icon.url = Some(interned.name());
            interned_images.insert(url, interned);
        }

        Ok(())
    }

    /// We return [Option<Icon>] so we can clear images with broken links
    /// (as some readers might not support them).
    fn icons_mut(&mut self) -> impl Iterator<Item = &mut Icon> {
        std::iter::once(&mut self.post.icon)
            .chain(self.replies.iter_mut().map(|r| &mut r.icon))
            .flatten()
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
