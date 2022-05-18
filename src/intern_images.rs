use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
};

use mime::Mime;

use crate::{
    types::{Icon, Thread},
    utils::mime_to_image_extension,
};

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
}

impl Thread {
    /// We return [Option<Icon>] so we can clear images with broken links
    /// (as some readers might not support them).
    fn icons_mut(&mut self) -> impl Iterator<Item = &mut Icon> {
        std::iter::once(&mut self.post.icon)
            .chain(self.replies.iter_mut().map(|r| &mut r.icon))
            .flatten()
    }

    pub async fn intern_images(
        &mut self,
    ) -> Result<BTreeMap<String, InternedImage>, Box<dyn Error>> {
        let mut interned_images: BTreeMap<String, InternedImage> = BTreeMap::new();

        let mut skip: BTreeSet<u64> = BTreeSet::default();

        for icon in self.icons_mut() {
            if skip.contains(&icon.id) {
                continue;
            }
            if let Err(e) = process_icon(icon, &mut interned_images).await {
                println!(
                    "Was unable to retrieve icon {}, the original url will be inlined.",
                    icon.id
                );
                println!("{e:?}");
                skip.insert(icon.id);
            }
        }

        Ok(interned_images)
    }
}

async fn process_icon(
    icon: &mut Icon,
    interned_images: &mut BTreeMap<String, InternedImage>,
) -> Result<(), Box<dyn Error>> {
    let id: usize = icon.id.try_into().unwrap();

    if !interned_images.contains_key(&icon.url) {
        let (mime, data) = icon.retrieve_cached(false).await?;
        let image = InternedImage {
            id,
            original_url: icon.url.clone(),
            mime,
            data,
        };
        icon.url = image.name();
        interned_images.insert(icon.url.clone(), image);
    }

    Ok(())
}
