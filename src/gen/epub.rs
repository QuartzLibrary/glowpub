use std::{collections::BTreeSet, error::Error};

use chrono::DateTime;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use rand::{Rng, SeedableRng};
use uuid::Uuid;

use crate::{
    types::{Continuity, Section, User},
    Board, Post, Reply, Thread,
};

use super::{
    author_names, raw_content_page, raw_copyright_page, raw_title_page, transform, Options, STYLE,
};

impl Continuity {
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

        let authors: Vec<User> = self.authors();

        // Metadata
        for author in &authors {
            builder.metadata("author", &author.username)?;
        }
        builder.metadata("title", &self.board.name)?;
        if let Some(created_at) = self.created_at() {
            builder.set_publication_date(created_at);
        }
        if let Some(tagged_at) = self.tagged_at() {
            builder.set_modified_date(tagged_at);
        }
        builder.set_uuid(self.uuid());

        // CSS
        builder.stylesheet(STYLE.as_bytes())?;

        // Cover Image
        builder.add_cover_image(
            "cover.png",
            super::cover::image(&self.board.name, &authors).as_slice(),
            mime::IMAGE_PNG.to_string(),
        )?;

        // Title
        builder.add_content(
            EpubContent::new("title.xhtml", self.to_title_page().as_bytes())
                .title("Title")
                .reftype(ReferenceType::TitlePage),
        )?;

        // We either have:
        // - Book
        //   -Board (level 0)
        //     - Section (level 1)
        //       - Post/Thread (level 2)
        //         - Reply chunk (level 3)
        //     - Sectionless [optional]
        //       - Post/Thread (level 2)
        //         - Reply chunk (level 3)
        // Or
        // - Book
        //   -Board (level 0)
        //     -Sectionless [invisible]
        //     - Post/Thread (level 1)
        //       - Reply chunk (level 2)

        let (sections, sectionless_threads) = self.sections();

        for (section, threads) in &sections {
            let Section { id, name, .. } = section;

            let section_path = format!("section_{id}");

            // Section intro
            builder
                .add_content(
                    EpubContent::new(
                        format!("{section_path}_title.xhtml"),
                        section.to_title_page(threads).as_bytes(),
                    )
                    .title(name)
                    .reftype(ReferenceType::TitlePage)
                    .level(1),
                )
                .unwrap();

            for thread in threads {
                thread.include(&section_path, 2, &mut builder, options)?;
            }
        }

        if !sectionless_threads.is_empty() && !sections.is_empty() {
            // Sectionless intro
            builder
                .add_content(
                    EpubContent::new(
                        "sectionless_title.xhtml",
                        Section::sectionless_title_page(&sectionless_threads).as_bytes(),
                    )
                    // .title("Sectionless Threads")
                    .reftype(ReferenceType::TitlePage)
                    .level(1),
                )
                .unwrap();
        }
        for thread in sectionless_threads {
            thread.include(
                "sectionless",
                if sections.is_empty() { 1 } else { 2 },
                &mut builder,
                options,
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
            self.board.id.to_be_bytes(),
            self.created_at()
                .as_ref()
                .map(DateTime::timestamp)
                .unwrap_or(0)
                .to_be_bytes(),
            self.tagged_at()
                .as_ref()
                .map(DateTime::timestamp)
                .unwrap_or(0)
                .to_be_bytes(),
        ];
        let seed: Vec<_> = seed.iter().flatten().copied().collect();
        let uuid = rand::rngs::StdRng::from_seed(seed.try_into().unwrap()).gen();
        Uuid::from_u128(uuid)
    }
}

impl Continuity {
    fn to_title_page(&self) -> String {
        let Board { id, name, .. } = &self.board;

        let authors = self.authors();
        let author_names = author_names(&authors);
        let author_ids: Vec<u64> = authors.iter().map(|user| user.id).collect();

        let thread_count = self.threads.len();

        let title_page = format!(
            r##"

        <div class="title-page">
            <h1 board-id="{id}">{name}</h1>
            <h2 glowfic-ids="{author_ids:?}">by {author_names}</h2>
            <p>[{thread_count} threads]</p>
        </div>

        "##
        );

        wrap_xml(name, &title_page)
    }
    fn to_copyright_page(&self) -> String {
        let Board { id, name, .. } = &self.board;

        let authors = self.authors();
        let author_names = author_names(&authors);
        let author_ids: Vec<u64> = authors.iter().map(|user| user.id).collect();

        let copyright = format!(
            r##"
    
        <div class="copyright-page">
            <h3>This was</h3>
            <h1 board-id="{id}">{name}</h1>
            <h2 glowfic-ids="{author_ids:?}">by {author_names}</h2>
            <h3 class="board" board-id="{id}">in {name}</h3>
    
            © {author_names}
        </div>
    
        "##
        );

        wrap_xml(&format!("{name} - Copyright"), &copyright)
    }
}
impl Section {
    fn to_title_page(&self, threads: &[&Thread]) -> String {
        let Section { id, name, .. } = self;
        let authors: Vec<User> = threads
            .iter()
            .flat_map(|t| t.post.authors.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let author_names = author_names(&authors);
        let author_ids: Vec<u64> = authors.iter().map(|user| user.id).collect();
        let thread_count = threads.len();

        let title_page = format!(
            r##"
    
        <div class="title-page">
            <h1 section-id="{id}">{name}</h1>
            <h2 glowfic-ids="{author_ids:?}">by {author_names}</h2>
            <p>[{thread_count} threads]</p>
        </div>
    
        "##
        );

        wrap_xml(name, &title_page)
    }
    fn sectionless_title_page(threads: &[&Thread]) -> String {
        let name = "Unsectioned Posts";
        let authors: Vec<User> = threads
            .iter()
            .flat_map(|t| t.post.authors.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let author_names = author_names(&authors);
        let author_ids: Vec<u64> = authors.iter().map(|user| user.id).collect();
        let thread_count = threads.len();

        let title_page = format!(
            r##"
    
        <div class="title-page">
            <h1>{name}</h1>
            <h2 glowfic-ids="{author_ids:?}">by {author_names}</h2>
            <p>[{thread_count} threads]</p>
        </div>
    
        "##
        );

        wrap_xml(name, &title_page)
    }
}
impl Thread {
    fn include(
        &self,
        prefix: &str,
        base_level: i32,
        builder: &mut EpubBuilder<ZipLibrary>,
        options: Options,
    ) -> Result<(), Box<dyn Error>> {
        let Post { id: post_id, .. } = self.post;

        let post_path = format!("post_{post_id}");

        // Post title
        builder
            .add_content(
                EpubContent::new(
                    format!("{prefix}_{post_path}_title.xhtml"),
                    self.to_title_page().as_bytes(),
                )
                .title(&self.post.subject)
                .reftype(ReferenceType::TitlePage)
                .level(base_level),
            )
            .unwrap();

        // Description
        builder
            .add_content(
                EpubContent::new(
                    format!("{prefix}_{post_path}_description.xhtml"),
                    self.description_page(options).as_bytes(),
                )
                // .title("Description") // No title to avoid cluttering the table of contents.
                .reftype(ReferenceType::Preface)
                .level(base_level + 1),
            )
            .unwrap();

        // Parts
        for (i, reply_page) in self.reply_pages(options).iter().enumerate() {
            builder
                .add_content(
                    EpubContent::new(
                        format!("{prefix}_{post_path}_part_{i}.xhtml"),
                        reply_page.as_bytes(),
                    )
                    // .title(format!("Part {i}")) // No title to avoid cluttering the table of contents.
                    .reftype(ReferenceType::Text)
                    .level(base_level + 1),
                )
                .unwrap();
        }

        Ok(())
    }
}

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
        builder.set_modified_date(self.post.tagged_at);
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
            .reftype(ReferenceType::Preface)
            .level(1),
        )?;

        // Parts
        for (i, reply_page) in self.reply_pages(options).iter().enumerate() {
            builder.add_content(
                EpubContent::new(format!("part_{i}.xhtml"), reply_page.as_bytes())
                    .title(format!("Part {i}"))
                    .reftype(ReferenceType::Text)
                    .level(1),
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
        let subject = &self.post.subject;
        wrap_xml(
            &format!("{subject} - Copyright"),
            &raw_copyright_page(&self.post),
        )
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
