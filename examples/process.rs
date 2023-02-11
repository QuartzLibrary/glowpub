use clap::Parser;
use std::path::PathBuf;

use glowfic_to_epub::Thread;

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
}

#[tokio::main]
async fn main() {
    let Args {
        post_id,
        use_cache,
        text_to_speech,
    } = Args::parse();

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
        std::fs::write(path, thread.to_single_html_page(text_to_speech)).unwrap();
    }

    {
        println!("Generating epub document...");

        let path = PathBuf::from(format!("./books/epub/{post_id}.epub"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, thread.to_epub(text_to_speech).await.unwrap()).unwrap();
    }
}
