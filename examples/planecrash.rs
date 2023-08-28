use clap::Parser;
use std::{collections::BTreeSet, path::PathBuf};

use glowpub::{cached::write_if_changed, gen::Options, Thread};

/// Board 215
/// Planecrash
pub const PLANECRASH: [&[u64]; 3] = [&MAIN, &SANDBOXES, &LECTURES];

/// Board section 703
/// Main planescrash section
pub const MAIN: [u64; 12] = [
    4582, // mad investor chaos and the woman of asmodeus
    5504, // some human relationships are less universal than others [follow up to 4582]
    5506, // take this report back and bring her a better report
    5508, // project lawful and their oblivious boyfriend
    5694, // my fun research project has more existential risk than I anticipated
    5930, // what the truth can destroy
    5977, // crisis of faith
    6075, // the woman of irori
    6132, // null action
    6334, // the meeting of their minds
    6480, // null action act ii: unact harder
    6827, // null action act iii: the consequences of my own nonactions
];

/// Board section 717
/// planecrash sandboxes
/// experimental doomthreads
pub const SANDBOXES: [u64; 5] = [
    6124, // dear abrogail
    6029, // it is a beautiful day in Cheliax and you are a horrible medianworld romance novel
    5880, // I reject your alternate reality and substitute my own [linked out from thread 5694 at reply 1789682]
    5778, // welcome to project lawful
    5775, // totally not evil
];

/// Board section 721
/// planecrash lectures
pub const LECTURES: [u64; 11] = [
    5785, // to hell with science [linked out from thread 5694 at reply 1777291]
    5826, // to earth with science
    5864, // the alien maths of dath ilan [linked out from thread 5694 at reply 1786765]
    6131, // flashback: this is not a threat
    5310, // kissing is not a human universal [linked out from thread 4582 at reply 1721818]
    5403, // sfw tldr kissing is not a human universal [linked out from thread 4582 at reply 1721818]
    5521, // tldr some human relationships [alternative to 5504]
    5610, // cheating is cuddleroom technique [linked out from thread 5508 at reply 1756345]
    5618, // sfw tldr cheating is cuddleroom technique [linked out from thread 5508 at reply 1756345]
    5638, // in another world we could have been trade partners [linked out from thread 5508 at reply 1760768]
    5671, // sfw tldr we could have been trade partners [linked out from thread 5508 at reply 1760768]
];

/// Download and process all glowfic posts in the planecrash series.
#[derive(Parser, Debug)]
struct Args {
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
        use_cache,
        text_to_speech,
        flatten_details,
    } = Args::parse();

    let options = Options {
        text_to_speech,
        flatten_details,
    };

    let mut threads = vec![];

    for id in PLANECRASH.into_iter().flatten().copied() {
        log::info!("Downloading post {id}");

        let thread = Thread::get_cached(id, !use_cache).await.unwrap().unwrap();

        log::info!("Downloaded post {id} - {}", &thread.post.subject);

        threads.push(thread);
    }

    let icons: BTreeSet<_> = threads
        .iter()
        .flat_map(|thread| thread.icons())
        .cloned()
        .collect();

    for icon in icons {
        if let Err(e) = icon.retrieve_cached(false).await {
            log::info!("{e:?}");
        }
    }

    for thread in threads {
        let post_id = thread.post.id;

        {
            log::info!("Generating html document {post_id}...");

            let path = PathBuf::from(format!("./books/html/{post_id}.html"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            write_if_changed(path, thread.to_single_html_page(options)).unwrap();
        }

        {
            log::info!("Generating epub document {post_id}...");

            let path = PathBuf::from(format!("./books/epub/{post_id}.epub"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            write_if_changed(path, thread.to_epub(options).await.unwrap()).unwrap();
        }
    }

    log::info!("Done")
}
