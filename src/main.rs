use clap::Parser;
use std::path::{Path, PathBuf};

use glowpub::{cached::write_if_changed, gen::Options, types::Continuity, Thread};

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
            Command::Post { options, .. } | Command::Board { options, .. } => *options,
        }
    }
}

#[derive(Debug, Clone, Copy, Parser)]
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

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let command = Command::parse();

    let CliOptions {
        use_cache,
        text_to_speech,
        flatten_details,
    } = command.options();

    let epub_options = Options {
        text_to_speech,
        flatten_details: match flatten_details.unwrap_or_default() {
            FlattenDetails::All | FlattenDetails::Mixed => true,
            FlattenDetails::None => false,
        },
    };
    let html_options = Options {
        text_to_speech,
        flatten_details: match flatten_details.unwrap_or_default() {
            FlattenDetails::All => true,
            FlattenDetails::None | FlattenDetails::Mixed => false,
        },
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

            log::info!("Generating html document...");
            let path = PathBuf::from(format!("./books/html/{post_id}.html"));
            write(path, thread.to_single_html_page(html_options));

            log::info!("Generating epub document...");
            let path = PathBuf::from(format!("./books/epub/{post_id}.epub"));
            write(path, thread.to_epub(epub_options).await.unwrap());
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

            for thread in continuity.threads {
                let post_id = thread.post.id;

                log::info!("Generating html document {post_id}...");
                let path = PathBuf::from(format!("./books/html/{post_id}.html"));
                write(path, thread.to_single_html_page(html_options));

                log::info!("Generating epub document {post_id}...");
                let path = PathBuf::from(format!("./books/epub/{post_id}.epub"));
                write(path, thread.to_epub(epub_options).await.unwrap());
            }
        }
        Command::Board {
            board_id,
            single_file: true,
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

            log::info!("Generating epub document {board_id}...");
            let path = PathBuf::from(format!("./books/epub/board_{board_id}.epub"));
            write(path, continuity.to_epub(epub_options).await.unwrap());
        }
    }

    log::info!("Done");
}

pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) {
    std::fs::create_dir_all(path.as_ref().parent().unwrap()).unwrap();
    write_if_changed(path, contents).unwrap();
}
