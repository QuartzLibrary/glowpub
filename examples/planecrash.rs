use clap::Parser;
use std::{collections::BTreeSet, path::PathBuf};

use glowfic_to_epub::Thread;

pub const IDS: [u64; 10] = [4503, 4582, 5504, 5506, 5508, 5694, 5775, 5778, 5880, 5930];

/// Download and process a all glowfic posts in the planecrash series.
#[derive(Parser, Debug)]
struct Args {
    /// Reuse already downloaded data. Images are always cached.
    #[clap(long)]
    use_cache: bool,
}

#[tokio::main]
async fn main() {
    let Args { use_cache } = Args::parse();

    let mut threads = vec![];

    for id in IDS {
        println!("Downloading post {id}");

        let thread = Thread::get_cached(id, !use_cache).await.unwrap().unwrap();

        println!("Downloaded post {id} - {}", &thread.post.subject);

        threads.push(thread);
    }

    let icons: BTreeSet<_> = threads
        .iter()
        .flat_map(|thread| thread.icons())
        .cloned()
        .collect();

    for icon in icons {
        if let Err(e) = icon.retrieve_cached(false).await {
            println!("{e:?}");
        }
    }

    for thread in threads {
        let post_id = thread.post.id;

        {
            println!("Generating html document {post_id}...");

            let path = PathBuf::from(format!("./books/html/{post_id}.html"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, thread.to_single_html_page()).unwrap();
        }

        {
            println!("Generating epub document {post_id}...");

            let path = PathBuf::from(format!("./books/epub/{post_id}.epub"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, &thread.to_epub().await.unwrap()).unwrap();
        }
    }

    println!("Done")
}
