use clap::Parser;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use glowpub::{
    api::BoardPosts,
    cached::write_if_changed,
    gen::Options,
    types::{Continuity, Section},
    Board, Thread,
};

const DEFAULT_OUTPUT_DIR: &str = "./books";

/// Download and process Glowfic posts into epub and html files.
#[derive(Debug, Parser)]
enum Command {
    /// Download and process a single post.
    Post {
        /// The id of the Glowfic post.
        /// Can be found in the URL: https://glowfic.com/posts/<id>
        post_id: u64,

        #[command(flatten)]
        options: CliOptions,
    },
    /// Download and process an entire board.
    Board {
        /// The id of the Glowfic board.
        /// Can be found in the URL: https://glowfic.com/boards/<id>
        board_id: u64,

        #[command(flatten)]
        options: CliOptions,

        /// If enabled, the board will be processed into a single epub file instead of being split by post.
        #[clap(long)]
        single_file: bool,
    },
}
impl Command {
    fn options(&self) -> CliOptions {
        match self {
            Command::Post { options, .. } | Command::Board { options, .. } => options.clone(),
        }
    }
}

#[derive(Debug, Clone, Parser)]
struct CliOptions {
    /// Reuse already downloaded data. Images are always cached.
    #[clap(long)]
    use_cache: bool,

    /// Simplify character and user names to improve text-to-speech output.
    #[clap(long)]
    text_to_speech: bool,

    /// <details> tags can be hard to use on e-readers, this option forces them to always seem open.
    ///
    /// (Under the hood, it replaces the <details> tag with a <blockquote>, and <summary> with <p>,
    /// it also preprends `▼ ` to the <summary> tag to make it similar to an open <details> tag.)
    #[clap(long)]
    flatten_details: Option<FlattenDetails>,

    /// When inlining the images into the epub file, this will convert all images into jpeg files.
    /// In general this will result in considerably smaller files if the images are not already jpegs.
    /// (Does not affect SVGs.)
    #[clap(long)]
    jpeg: bool,

    /// When inlining icons into the epub file, this will scale all icon images above the provided width down to that width.
    /// Defaults to "100" if no value is provided.
    /// (Does not affect SVGs or non-icon images.)
    #[clap(long)]
    resize_icons: Option<Option<u32>>,

    /// Output files in this directory (e.g. `--output-dir=~/glowfic`).
    /// Note that this can flood the directory if used with `board` but without `--single-file`.
    /// Files will be placed in format-specific subdirectories if this option is not set, or if `--output-format` is `both` (the default).
    #[clap(long)]
    output_dir: Option<PathBuf>,

    /// Determines which file-types are created by the program.
    #[clap(long, default_value_t = OutputFormat::default())]
    output_format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
enum FlattenDetails {
    /// The default option. No <details> tags will be flattened.
    #[default]
    None,
    /// All <details> tags will be flattened.
    All,
    /// Only <details> tags in epubs will be flattened.
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
enum OutputFormat {
    /// The default option. Both HTML and Epub files will be created.
    /// Output files will be placed in "html" and "epub" subdirectories, respectively.
    #[default]
    Both,
    /// Only Epub files will be created.
    Epub,
    /// Only HTML files will be created.
    Html,
    /// No output files will be created.
    None,
}
impl OutputFormat {
    fn epub(self) -> bool {
        matches!(self, Self::Epub | Self::Both)
    }
    fn html(self) -> bool {
        matches!(self, Self::Html | Self::Both)
    }
}
impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Both => write!(f, "both"),
            Self::Epub => write!(f, "epub"),
            Self::Html => write!(f, "html"),
            Self::None => write!(f, "none"),
        }
    }
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let command = Command::parse();

    let CliOptions {
        use_cache,
        text_to_speech,
        flatten_details,
        jpeg,
        resize_icons,
        output_dir,
        output_format,
    } = command.options();

    let resize_icons = resize_icons.map(|r| r.unwrap_or(100));

    let mut html_output_dir = output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT_DIR));
    let mut epub_output_dir = output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT_DIR));

    if output_format == OutputFormat::Both || output_dir.is_none() {
        html_output_dir = html_output_dir.join("html/");
        epub_output_dir = epub_output_dir.join("epub/");
    }

    let epub_options = Options {
        text_to_speech,
        flatten_details: match flatten_details.unwrap_or_default() {
            FlattenDetails::All | FlattenDetails::Mixed => true,
            FlattenDetails::None => false,
        },
        jpeg,
        resize_icons,
    };
    let html_options = Options {
        text_to_speech,
        flatten_details: match flatten_details.unwrap_or_default() {
            FlattenDetails::All => true,
            FlattenDetails::None | FlattenDetails::Mixed => false,
        },
        jpeg,
        resize_icons,
    };

    match command {
        Command::Post { post_id, .. } => {
            log::info!("Downloading post {post_id}");
            let thread = Thread::get_cached(post_id, !use_cache)
                .await
                .unwrap()
                .unwrap();
            log::info!("Downloaded post {post_id} - {}", &thread.post.subject);

            log::info!("Caching all the icons...");
            thread.cache_all_icons(false).await;

            let name = {
                let board = Board::get_cached(thread.post.board.id, !use_cache)
                    .await
                    .unwrap()
                    .unwrap();

                let board_posts = BoardPosts::get_all_cached(board.id, !use_cache)
                    .await
                    .unwrap()
                    .unwrap();

                thread_filename(
                    &thread,
                    &board,
                    board_posts.iter().map(|p| p.section.clone()),
                )
            };

            if output_format.html() {
                log::info!("Generating html document {name}...");
                let path = html_output_dir.join(format!("{name}.html"));
                write(path, thread.to_single_html_page(html_options));
            }

            if output_format.epub() {
                log::info!("Generating epub document {name}...");
                let path = epub_output_dir.join(format!("{name}.epub"));
                write(path, thread.to_epub(epub_options).await.unwrap());
            }
        }
        Command::Board {
            board_id,
            single_file: false,
            ..
        } => {
            log::info!("Downloading board/continuity {board_id}...");
            let continuity = Continuity::get_cached(board_id, !use_cache)
                .await
                .unwrap()
                .unwrap();
            log::info!(
                "Downloaded continuity {board_id} - {}",
                &continuity.board.name
            );

            log::info!("Caching all the icons...");
            continuity.cache_all_icons(false).await;

            for thread in &continuity.threads {
                let name = thread_filename(
                    thread,
                    &continuity.board,
                    continuity.threads.iter().map(|t| t.post.section.clone()),
                );

                if output_format.html() {
                    log::info!("Generating html document {name}...");
                    let path = html_output_dir.join(format!("{name}.html"));
                    write(path, thread.to_single_html_page(html_options));
                }

                if output_format.epub() {
                    log::info!("Generating epub document {name}...");
                    let path = epub_output_dir.join(format!("{name}.epub"));
                    write(path, thread.to_epub(epub_options).await.unwrap());
                }
            }
        }
        Command::Board {
            board_id,
            single_file: true,
            ..
        } => {
            if output_format.html() {
                log::warn!("HTML output is not supported in single-file mode.");
                if output_format == OutputFormat::Html {
                    return;
                }
            }

            log::info!("Downloading board/continuity {board_id}...");
            let continuity = Continuity::get_cached(board_id, !use_cache)
                .await
                .unwrap()
                .unwrap();
            log::info!(
                "Downloaded continuity {board_id} - {}",
                &continuity.board.name
            );

            log::info!("Caching all the icons...");
            continuity.cache_all_icons(false).await;

            let name = {
                let board_id = continuity.board.id;
                let name = slug::slugify(&continuity.board.name);
                format!("[{board_id}] {name}")
            };

            log::info!("Generating epub document {name}...");
            let path = epub_output_dir.join(format!("{name}.epub"));
            write(path, continuity.to_epub(epub_options).await.unwrap());
        }
    }

    log::info!("Done");
}

fn thread_filename(
    thread: &Thread,
    board: &Board,
    board_thread_sections: impl Iterator<Item = Option<Section>>,
) -> String {
    let board_folder = {
        let board_id = board.id;
        let board_name = slug::slugify(&board.name);
        format!("[{board_id}] {board_name}/")
    };

    let section_folder = thread
        .post
        .section
        .clone()
        .map(|Section { id, name, order }| {
            let width = Ord::max(board.board_sections.len().to_string().len(), 2);
            let name = slug::slugify(name);
            format!("Section #{order:0width$} [{id}] {name}/")
        })
        .unwrap_or_default();

    let post_name = {
        let post_id = thread.post.id;
        let post_subject = slug::slugify(&thread.post.subject);
        let post_order = {
            let same_section_count = board_thread_sections
                .filter(|s| *s == thread.post.section)
                .count();
            let width = Ord::max(same_section_count.to_string().len(), 2);
            let order = thread.post.section_order;
            format!("{order:0width$}")
        };
        format!("#{post_order} [{post_id}] {post_subject}")
    };

    format!("{board_folder}{section_folder}{post_name}")
}

pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) {
    std::fs::create_dir_all(path.as_ref().parent().unwrap()).unwrap();
    write_if_changed(path, contents).unwrap();
}
