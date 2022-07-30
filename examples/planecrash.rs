use clap::Parser;
use std::{collections::BTreeSet, path::PathBuf};
use slug::slugify;

use glowfic_to_epub::Thread;

// This is my (Robin Lee Powell)'s best guess as to the correct reading order, given that I haven't
// actually read most of it yet, and given that in several places the text itself tells you to jump
// out to a subthread and come back, which makes "reading order" a bit of a miss anyway.
pub const ITEMS: [(&str, u64); 25] = [
    ("01", 4582), // MAIN: mad investor chaos and the woman of asmodeus
    ("01-subthread-01", 5310), // SUBTHREAD: kissing is not a human universal
    ("01-subthread-01-sfw-tldr", 5403), // ALTERNATE SUBTHREAD: sfw tldr kissing is not a human universal
    ("02-and-03-alternate", 5521), // ALTERNATE, less disturbing, for "some human" and "take this report"
    ("02", 5504), // MAIN: some human relationships are less universal than others
    ("03", 5506), // MAIN: take this report back and bring her a better report
    ("04", 5508), // MAIN: project lawful and their oblivious boyfriend
    ("04-subthread-01", 5610), // SUBTHREAD: cheating is cuddleroom technique
    ("04-subthread-01-sfw-tldr", 5618), // ALTERNATE SUBTHREAD: sfw tldr cheating is cuddleroom technique
    ("04-subthread-02", 5638), // SUBTHREAD: in another world we could have been trade partners
    ("04-subthread-02-sfw-tldr", 5671), // ALTERNATE SUBTHREAD: sfw tldr we could have been trade partners
    ("04-sandbox-01", 5775), // SANDBOX: totally not evil
    ("04-sandbox-02", 5778), // SANDBOX: welcome to project lawful
    ("05", 5694), // MAIN: my fun research project has more existential risk than I anticipated
    ("05-subthread-01", 5785), // SUBTHREAD / LECTURE: to hell with science
    ("05-subthread-02", 5826), // SUBTHREAD SORT OF?: to earth with science
    ("05-subthread-03", 5864), // SUBTHREAD / LECTURE: the alien maths of dath ilan
    ("05-sandbox-01", 5880), // SANDBOX: I reject your alternate reality and substitute my own
    ("06", 5930), // MAIN: what the truth can destroy
    ("06-sandbox-01", 6029), // SANDBOX: it is a beautiful day in Cheliax and you are a horrible medianworld romance novel
    ("07", 5977), // MAIN: crisis of faith
    ("08", 6075), // MAIN: the woman of irori
    ("08-sandbox-01", 6124), // SANDBOX: dear abrogail
    ("09", 6131), // MAIN: flashback: this is not a threat
    ("10", 6132), // MAIN: null action
];

/// Download and process a all glowfic posts in the planecrash series.
#[derive(Parser, Debug)]
struct Args {
    /// Reuse already downloaded data. Images are always cached.
    #[clap(long)]
    use_cache: bool,

    /// Reformat the author/character bits to make for easier TTS listening.
    #[clap(long)]
    for_tts: bool,
}

#[tokio::main]
async fn main() {
    let Args { use_cache, for_tts } = Args::parse();

    let mut threads = vec![];

    for (preamble, id) in ITEMS {
        println!("Downloading post {id}");

        let thread = Thread::get_cached(id, !use_cache).await.unwrap().unwrap();

        println!("Downloaded post {id} - {}", &thread.post.subject);

        threads.push((preamble, thread));
    }

    let icons: BTreeSet<_> = threads
        .iter()
        .flat_map(|(_, thread)| thread.icons())
        .cloned()
        .collect();

    for icon in icons {
        if let Err(e) = icon.retrieve_cached(false).await {
            println!("{e:?}");
        }
    }

    for (preamble, thread) in threads {
        let post_id = thread.post.id;
        let title_slug = slugify(&thread.post.subject);

        {
            println!("Generating html document {preamble}_{post_id}_{title_slug}...");

            let path = PathBuf::from(format!("./books/html/{preamble}_{post_id}_{title_slug}.html"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, thread.to_single_html_page(for_tts)).unwrap();
        }

        {
            println!("Generating epub document {preamble}_{post_id}_{title_slug}...");

            let path = PathBuf::from(format!("./books/epub/{preamble}_{post_id}_{title_slug}.epub"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, &thread.to_epub(for_tts).await.unwrap()).unwrap();
        }
    }

    println!("Done")
}
