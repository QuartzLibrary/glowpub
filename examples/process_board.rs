use clap::Parser;
use std::{
    path::PathBuf,
    collections::{
        BTreeSet,
        BTreeMap,
    },
};
use slug::slugify;
use count_digits::CountDigits;

use glowpub::{
    cached::write_if_changed,
    gen::{
        Options, STYLE, author_names,
        epub::wrap_xml,
    },
    api::{Replies, BoardPosts},
    types::{Section, Post, PostInBoard},
    Board, Thread
};

use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use rand::{Rng, SeedableRng};
use uuid::Uuid;

/// Download and process a Glowfic board.
#[derive(Parser, Debug)]
struct Args {
    /// The id of the Glowfic board.
    /// Can be found in the URL: https://glowfic.com/boards/<id>
    board_id: u64,

    /// Reuse already downloaded data. Images are always cached.
    #[clap(long)]
    use_cache: bool,

    /// Simplify character and user names to improve text-to-speech output.
    #[clap(long)]
    text_to_speech: bool,

    /// Details tags can be hard to use on e-readers, this option forces them to always seem open.
    ///
    /// (Under the hood, it replaces the `details` tag with a `blockquote` and `summary` with `p`,
    /// it also preprends `▼ ` to the `summary` tag to make it similar to an open details tag.)
    #[clap(long)]
    flatten_details: bool,
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let Args {
        board_id,
        use_cache,
        text_to_speech,
        flatten_details,
    } = Args::parse();

    let options = Options {
        text_to_speech,
        flatten_details,
    };

    log::info!("Downloading board {board_id}");

    // get board; make sectioned_threads vec (Vec<(Section, Vec<Thread>, BTreeSet<User>)>).
    // probably deserves its own datatype, to be honest.
    let board = Board::get(board_id).await.unwrap().unwrap();
    let mut sections = board.board_sections;
    let num_sections = match u64::try_from(sections.len()) {
        Ok(number) => number,
        Err(_error) => u64::MAX,
    };
    // some boards have threads that aren't in any section, so we need a section for those, too
    sections.push(Section::null(num_sections));

    let mut sectioned_threads = vec![];
    for section in &sections {
        sectioned_threads.push((section, vec![], BTreeSet::new()));
    }
    //sort sections by order
    sectioned_threads.sort_by(|a, b| a.0.order.cmp(&b.0.order));

    log::info!("Fetching list of posts in board.");
    // get BoardPosts (all posts in board)
    // notably these *don't* have content, so we'll *also* need to fetch the posts individually.
    // don't look at me, that's just how the api works.
    let board_posts = BoardPosts::get_all_cached(board_id, !use_cache).await.unwrap().unwrap();
    let thread_count = &board_posts.len();
    let mut first_post: Option<PostInBoard> = None;
    let mut last_post: Option<PostInBoard> = None;

    // over here we'll actually get all of the threads
    let mut i = 0;
    let mut threads = vec![];
    for board_post in &board_posts {
        // I could totally have gotten these two in a less messy-looking way, but that'd've meant two more iterations!
        // they don't actually take very long to iterate through though, so I should probably make it less messy later.
        if match &first_post {
            Some(post) => board_post.created_at < post.created_at,
            None => true,
        } {
            first_post = Some(board_post.clone());
        }
        if match &last_post {
            Some(post) => board_post.tagged_at > post.tagged_at,
            None => true,
        } {
            last_post = Some(board_post.clone());
        }

        let board_post_id = board_post.id;
        i += 1;
        log::info!("Downloading post {board_post_id} ({i} of {thread_count}).");

        let post = Post::get_cached(board_post.id, !use_cache).await.unwrap().unwrap();
        let replies = Replies::get_all_cached(post.id, !use_cache).await.unwrap().unwrap();
        threads.push(Thread { post, replies });
    }
    let first_created_at = first_post.unwrap().created_at;
    let last_tagged_at = last_post.unwrap().tagged_at;


    // shamelessly stolen from the planecrash script
    log::info!("Downloading icons...");
    let icons: BTreeSet<_> = threads
        .iter()
        .flat_map(|thread| thread.icons())
        .cloned()
        .collect();
    for icon in icons {
        if let Err(e) = icon.retrieve_cached(!use_cache).await {
            log::info!("{e:?}");
        }
    }


    // sectioned_threads has authors too, but those are stored per-section, whereas this is for the whole board
    let mut authors = BTreeSet::new();
    let mut interned_images = BTreeMap::new();
    
    log::info!("Mapping threads:");
    i = 0;
    for thread in &mut threads {
        let section_id = match thread.post.section {
            Some(ref section) => section.id,
            None => 0,
        };
        let section = sectioned_threads.iter_mut().find(|r| r.0.id == section_id).unwrap();

        //authors
        authors.append(&mut BTreeSet::from_iter(thread.post.authors.clone().into_iter()));
        section.2.append(&mut BTreeSet::from_iter(thread.post.authors.clone().into_iter()));

        //images
        let thread_id = thread.post.id;
        i += 1;
        log::info!("Retrieving images for thread {thread_id} ({i} of {thread_count})...");
        // for some reason this takes a while for long threads, even if the images are already cached. might wanna look into that.
        interned_images.append(&mut thread.intern_images().await.unwrap());
    
        // this bit needs to go after interning the images, or else it won't use the saved images.
        section.1.push(thread);
    }

    let board_name = board.name;
    let board_title_slug = slugify(&board_name);
    let section_order_max_digits = sections.as_slice().last().unwrap().order.count_digits();

    // most of this is stolen from the epub library, and I had to make a lot of previously-private stuff public to make it work.
    // this is *far* from clean code, but with how hacky getting a whole board is, I figured it'd be better to keep the hackiness in the example.
    log::info!("Generating epub:");
    let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    
    log::info!("Saving images...");
    for (_, image) in interned_images {
        builder.add_resource(image.name(), image.data.as_slice(), image.mime.to_string()).unwrap();
    }

    // Metadata
    log::info!("Generating metadata.");
    for author in &authors {
        builder.metadata("author", &author.username).unwrap();
    }
    builder.metadata("title", &board_name).unwrap();
    builder.set_publication_date(first_created_at);
    builder.set_modified_date(last_tagged_at);
    let board_uuid = 
    {
        let seed: [[u8; 8]; 4] = [
            *b"glowfic!",
            board.id.to_be_bytes(),
            first_created_at.timestamp().to_be_bytes(),
            last_tagged_at.timestamp().to_be_bytes(),
        ];
        let seed: Vec<_> = seed.iter().flatten().copied().collect();
        let uuid = rand::rngs::StdRng::from_seed(seed.try_into().unwrap()).gen();
        Uuid::from_u128(uuid)
    };
    builder.set_uuid(board_uuid);

    // CSS
    builder.stylesheet(STYLE.as_bytes()).unwrap();

    let authors_vec = &Vec::from_iter(authors.into_iter());

    // Cover Image
    builder.add_cover_image(
        "cover.png",
        glowpub::gen::cover::image(&board_name, authors_vec).as_slice(),
        mime::IMAGE_PNG.to_string(),
    ).unwrap();

    let board_author_names = author_names(authors_vec);
    let author_ids: String = format!(
        "{:?}",
        authors_vec.iter().map(|user| user.id).collect::<Vec<_>>()
    );

    log::info!("Generating contents:");
    
    // Title
    // I wanna say that the titles and copyright pages are ugly, but I needed to give them custom content anyway, so meh.
    {
        builder.add_content(
            EpubContent::new("title.xhtml", wrap_xml(
                &board_name,
                &format!(
                    r##"
            
                <div class="title-page">
                    <h1 class="title" board-id="{board_id}">{board_name}</h1>
                    <h2 class="authors" glowfic-ids="{author_ids}">by {board_author_names}</h2>
                    <p class="thread-count">[{thread_count} threads]</p>
                </div>
            
                "##
                ),
            ).as_bytes())
                .title("Title")
                .reftype(ReferenceType::TitlePage),
        ).unwrap();
    }
    

    for thread_section in &mut sectioned_threads {
        // sort posts by section_order
        thread_section.1.sort_by(|a, b| a.post.section_order.cmp(&b.post.section_order));
        // generate a bunch of section-specific data for later
        let section_slug = slugify(&thread_section.0.name);
        let section_order = format!("{:0width$}", thread_section.0.order.to_string(), width = section_order_max_digits);
        let thread_order_max_digits = match thread_section.1.as_slice().last() {
            Some(thread) => thread,
            None => continue,
        }.post.section_order.count_digits();
        let section_name = &thread_section.0.name;
        let section_id = thread_section.0.id;
        let thread_count = thread_section.1.len().to_string();
        let section_authors = &Vec::from_iter(thread_section.2.clone().into_iter());
        let section_author_names = author_names(section_authors);
        let section_author_ids: String = format!(
            "{:?}",
            section_authors.iter().map(|user| user.id).collect::<Vec<_>>()
        );

        // if there are sections in the board, add section titles and make all section threads nest under their section
        if num_sections > 0 {
            builder.add_content(
                EpubContent::new(format!("{section_id}_{section_slug}_title.xhtml"), wrap_xml(
                    &section_name,
                    &format!(
                        r##"
                
                    <div class="title-page">
                        <h1 class="title" section-id="{section_id}">{section_name}</h1>
                        <h2 class="authors" glowfic-ids="{section_author_ids}">by {section_author_names}</h2>
                        <p class="thread-count">[{thread_count} threads]</p>
                    </div>
                
                    "##
                    ),
                ).as_bytes())
                    .title(section_name)
                    .reftype(ReferenceType::TitlePage),
            ).unwrap();
        }
        let toc_level = match num_sections {
            0 => 1,
            1.. => 2,
        };

        for thread in thread_section.1.iter() {
            let post_id = thread.post.id;
            let post_title_slug = slugify(&thread.post.subject);

            // HTML
            {
                let post_order = format!("{:0>width$}", thread.post.section_order.to_string(), width = thread_order_max_digits);
                let post_filename = format!("{post_order} {post_id} - {post_title_slug}");

                log::info!("Generating html document \"{post_filename}\"...");

                let board_path = format!("{board_id} {board_title_slug}");
                let post_path = match num_sections {
                    0 => format!("{board_path}/{post_filename}"),
                    1.. => format!("{board_path}/{section_order} {section_slug}/{post_filename}"),
                };

                let path = PathBuf::from(format!("./books/html/{post_path}.html"));
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                write_if_changed(path, thread.to_single_html_page(options)).unwrap();
            }
            
            // Epub
            {
                log::info!("Generating epub content for post {post_id}...");
                // I was gonna make the threads use a nested folder structure in the epub, but it made accessing stored images a pain.
                let path = format!("{section_id}_{section_slug}_{post_id}_{post_title_slug}");
                let post_title = &thread.post.subject;

                // Title
                builder.add_content(
                    EpubContent::new(
                        format!("{path}_title.xhtml"),
                        thread.to_title_page().as_bytes()
                    )
                    .title(post_title)
                    .reftype(ReferenceType::TitlePage)
                    .level(toc_level),
                ).unwrap();

                // no title on the description or sections items. it clutters the table of contents.
                // Description
                builder.add_content(
                    EpubContent::new(
                        format!("{path}_description.xhtml"),
                        thread.description_page(options).as_bytes(),
                    )
                    .reftype(ReferenceType::Preface)
                    .level(toc_level + 1),
                ).unwrap();

                // Sections
                for (i, reply_page) in thread.reply_pages(options).iter().enumerate() {
                    builder.add_content(
                        EpubContent::new(
                            format!("{path}_section_{i}.xhtml"),
                            reply_page.as_bytes()
                        )
                        .reftype(ReferenceType::Text)
                        .level(toc_level + 1),
                    ).unwrap();
                }
            }
        }
    }

    // Copyright
    builder.add_content(
        EpubContent::new(
            "copyright.xhtml",
            wrap_xml(
                &board_name,
                &format!(
                    r##"
            
                <div class="copyright-page">
                    <h3>This was</h3>
                    <h1 class="title" post-id="{board_id}">{board_name}</h1>
                    <h2 class="authors" glowfic-ids="{author_ids}">by {board_author_names}</h2>
                    <h3 class="board" board-id="{board_id}">in {board_name}</h3>
            
                    © {board_author_names}
                </div>
            
                "##
                )
            ).as_bytes()
        )
        .title("Copyright")
        .reftype(ReferenceType::Copyright),
    ).unwrap();

    // Saving file
    let epub_filename = format!("glowfic board {board_id} {board_name}");
    log::info!("Saving epub document \"{epub_filename}\"...");

    let mut file: Vec<u8> = vec![];
    builder.generate(&mut file).unwrap();

    let path = PathBuf::from(format!("./books/epub/{epub_filename}.epub"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    write_if_changed(path, file).unwrap();

    log::info!("Done");
}
