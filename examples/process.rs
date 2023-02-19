use clap::Parser;
use std::path::PathBuf;

use glowpub::{cached::write_if_changed, gen::Options, Thread};

/// Download and process a Glowfic post.
#[derive(Parser, Debug)]
struct Args {
    /// The id of the Glowfic post.
    /// Can be found in the URL: https://glowfic.com/posts/<id>
    post_id: u64,

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
    let Args {
        post_id,
        use_cache,
        text_to_speech,
        flatten_details,
    } = Args::parse();

    let options = Options {
        text_to_speech,
        flatten_details,
    };

    println!("Downloading post {post_id}");

    let thread = Thread::get_cached(post_id, !use_cache)
        .await
        .unwrap()
        .unwrap();

    println!("Downloaded post {post_id} - {}", &thread.post.subject);

    {
        println!("Caching all the icons...");

        thread.cache_all_icons(false).await;
    }

    {
        println!("Generating html document...");

        let path = PathBuf::from(format!("./books/html/{post_id}.html"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        write_if_changed(path, thread.to_single_html_page(options)).unwrap();
    }

    {
        println!("Generating epub document...");

        let path = PathBuf::from(format!("./books/epub/{post_id}.epub"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        write_if_changed(path, thread.to_epub(options).await.unwrap()).unwrap();
    }
}
