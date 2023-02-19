use std::error::Error;

use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use rand::{Rng, SeedableRng};
use uuid::Uuid;

use crate::{Reply, Thread};

use super::{raw_content_page, raw_copyright_page, raw_title_page, transform, Options, STYLE};

impl Thread {
    pub async fn to_epub(&self, options: Options) -> Result<Vec<u8>, Box<dyn Error>> {
        self.clone().as_epub(options).await
    }
    async fn as_epub(&mut self, options: Options) -> Result<Vec<u8>, Box<dyn Error>> {
        let interned_images = self.intern_images().await?;

        let mut builder = self.core_epub(options)?;

        // Images
        for (_, image) in interned_images {
            builder.add_resource(image.name(), image.data.as_slice(), image.mime.to_string())?;
        }

        let mut file: Vec<u8> = vec![];
        builder.generate(&mut file)?;

        Ok(file)
    }

    pub fn to_epub_remote_images(&self, options: Options) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut builder = self.core_epub(options)?;

        let mut file: Vec<u8> = vec![];
        builder.generate(&mut file)?;

        Ok(file)
    }

    fn core_epub(&self, options: Options) -> Result<EpubBuilder<ZipLibrary>, Box<dyn Error>> {
        let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

        // Metadata
        for author in &self.post.authors {
            builder.metadata("author", &author.username)?;
        }
        builder.metadata("title", &self.post.subject)?;
        builder.set_publication_date(self.post.created_at);
        builder.set_last_modified_date(self.post.tagged_at);
        builder.set_uuid(self.uuid());

        // CSS
        builder.stylesheet(STYLE.as_bytes())?;

        // Cover Image
        builder.add_cover_image(
            "cover.png",
            super::cover::image(&self.post.subject, &self.post.authors).as_slice(),
            mime::IMAGE_PNG.to_string(),
        )?;

        // Title
        builder.add_content(
            EpubContent::new("title.xhtml", self.to_title_page().as_bytes())
                .title("Title")
                .reftype(ReferenceType::TitlePage),
        )?;

        // Description
        builder.add_content(
            EpubContent::new(
                "description.xhtml",
                self.description_page(options).as_bytes(),
            )
            .title("Description")
            .reftype(ReferenceType::Preface),
        )?;

        // Sections
        for (i, reply_page) in self.reply_pages(options).iter().enumerate() {
            builder.add_content(
                EpubContent::new(format!("section_{i}.xhtml"), reply_page.as_bytes())
                    .title(format!("Section {i}"))
                    .reftype(ReferenceType::Text),
            )?;
        }

        // Copyright
        builder.add_content(
            EpubContent::new("copyright.xhtml", self.to_copyright_page().as_bytes())
                .title("Copyright")
                .reftype(ReferenceType::Copyright),
        )?;

        Ok(builder)
    }

    fn uuid(&self) -> Uuid {
        let seed: [[u8; 8]; 4] = [
            *b"glowfic!",
            self.post.id.to_be_bytes(),
            self.post.created_at.timestamp().to_be_bytes(),
            self.post.tagged_at.timestamp().to_be_bytes(),
        ];
        let seed: Vec<_> = seed.iter().flatten().copied().collect();
        let uuid = rand::rngs::StdRng::from_seed(seed.try_into().unwrap()).gen();
        Uuid::from_u128(uuid)
    }
}

impl Thread {
    fn to_title_page(&self) -> String {
        wrap_xml(
            &self.post.subject,
            &raw_title_page(&self.post, self.replies.len()),
        )
    }
    fn description_page(&self, options: Options) -> String {
        let subject = &self.post.subject;
        wrap_xml(
            &format!("{subject} - Description"),
            &raw_content_page(&[self.post.content_block(options)]),
        )
    }
    fn reply_pages(&self, options: Options) -> Vec<String> {
        let subject = &self.post.subject;
        let mut pages = vec![];

        let replies: Vec<String> = self
            .replies
            .iter()
            .map(|reply| Reply::content_block(reply, options))
            .collect();
        for (i, chunk) in replies.chunks(30).enumerate() {
            pages.push(wrap_xml(
                &format!("{subject} - Section {i}"),
                &raw_content_page(chunk),
            ));
        }

        pages
    }
    fn to_copyright_page(&self) -> String {
        wrap_xml(&self.post.subject, &raw_copyright_page(&self.post))
    }
}

fn wrap_xml(subject: &str, content: &str) -> String {
    let content = transform::html_to_xml(content);

    format!(
        r##"<?xml version='1.0' encoding='utf-8'?>
<html xmlns="http://www.w3.org/1999/xhtml" lang="en" xml:lang="en">
    <head>
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <meta name="theme-color" content="#000000"/>
        <title>{subject}</title>
        <meta http-equiv="Content-Type" content="text/html; charset=utf-8"/>
        <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
    </head>
    <body>

        {content}

    </body>
</html>

    "##
    )
}
