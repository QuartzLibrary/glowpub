use std::{
    collections::BTreeSet,
    error::Error,
    path::{Path, PathBuf},
};

use mime::Mime;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    api::{GlowficError, Replies, BoardPosts, Threads, SectionedThreads},
    types::{Icon, Thread, Section, PostInBoard},
    utils::{extension_to_image_mime, mime_to_image_extension},
    Board, Post, Reply,
};

const CACHE_ROOT: &str = "./cache";

impl Board {
    fn cache_key(id: u64) -> PathBuf {
        format!("{CACHE_ROOT}/boards/{id}.json").into()
    }

    pub async fn get_cached(
        id: u64,
        invalidate_cache: bool,
    ) -> Result<Result<Self, Vec<GlowficError>>, Box<dyn Error>> {
        get_cached_glowfic(&Self::url(id), &Self::cache_key(id), invalidate_cache).await
    }
}

impl Post {
    fn cache_key(id: u64) -> PathBuf {
        format!("{CACHE_ROOT}/posts/{id}/post.json").into()
    }

    pub async fn get_cached(
        id: u64,
        invalidate_cache: bool,
    ) -> Result<Result<Self, Vec<GlowficError>>, Box<dyn Error>> {
        get_cached_glowfic(&Self::url(id), &Self::cache_key(id), invalidate_cache).await
    }
}

impl Replies {
    fn cache_key(id: u64) -> PathBuf {
        format!("{CACHE_ROOT}/posts/{id}/replies.json").into()
    }

    pub async fn get_all_cached(
        id: u64,
        invalidate_cache: bool,
    ) -> Result<Result<Vec<Reply>, Vec<GlowficError>>, Box<dyn Error>> {
        let cache_path = Self::cache_key(id);

        if !invalidate_cache {
            if let Ok(data) = std::fs::read(&cache_path) {
                let parsed: Result<Self, Vec<GlowficError>> =
                    serde_json::from_slice(&data).unwrap();
                if let Ok(replies) = parsed {
                    return Ok(Ok(replies.0));
                }
            }
        }

        let response = Self::get_all(id).await?;

        std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        write_if_changed(&cache_path, serde_json::to_vec_pretty(&response).unwrap()).unwrap();

        Ok(response)
    }
}

impl Icon {
    fn cache_key(id: u64, extension: &str) -> PathBuf {
        // Note: names starting with a number can be problematic in epubs.
        format!("{CACHE_ROOT}/images/glowfic_{id}.{extension}").into()
    }

    pub async fn retrieve_cached(
        &self,
        invalidate_cache: bool,
    ) -> Result<(Mime, Vec<u8>), Box<dyn Error>> {
        let Self { id, url, .. } = self;

        if !invalidate_cache {
            let files: Vec<_> = {
                let blob_path = Self::cache_key(*id, "*");
                let files: Vec<_> = glob::glob(blob_path.to_str().unwrap()).unwrap().collect();

                // We expect the blob to ever only match at most 1 file.
                assert!(files.len() <= 1);

                files
            };

            if let Some(Ok(path)) = files.first() {
                let data = std::fs::read(path).unwrap();

                let extension = path.extension().unwrap().to_str().unwrap();
                if let Some(mime) = extension_to_image_mime(extension) {
                    return Ok((mime, data));
                }
            }
        }

        log::info!("Downloading icon {id} from {url}");

        let (mime, data) = self.retrieve().await?;
        let extension = mime_to_image_extension(&mime).ok_or(format!("Invalid mime: {mime}"))?;

        let cache_path = Self::cache_key(*id, &extension);
        std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        write_if_changed(cache_path, &data).unwrap();

        Ok((mime, data))
    }
}

impl Thread {
    pub async fn get_cached(
        id: u64,
        invalidate_cache: bool,
    ) -> Result<Result<Thread, Vec<GlowficError>>, Box<dyn Error>> {
        let post = match Post::get_cached(id, invalidate_cache).await? {
            Ok(post) => post,
            Err(errors) => return Ok(Err(errors)),
        };
        let replies = match Replies::get_all_cached(id, invalidate_cache).await? {
            Ok(replies) => replies,
            Err(errors) => return Ok(Err(errors)),
        };

        Ok(Ok(Self { post, replies }))
    }
    pub async fn cache_all_icons(&self, invalidate_cache: bool) {
        let icons: BTreeSet<_> = self.icons().collect();

        for icon in icons {
            if let Err(e) = icon.retrieve_cached(invalidate_cache).await {
                log::info!("{e:?}");
            }
        }
    }
}

impl BoardPosts {
    fn board_cache_key(board_id: u64) -> PathBuf {
        format!("{CACHE_ROOT}/boards/{board_id}/posts.json").into()
    }

    pub async fn board_get_all_cached(
        board_id: u64,
        invalidate_cache: bool,
    ) -> Result<Result<Vec<PostInBoard>, Vec<GlowficError>>, Box<dyn Error>> {
        let cache_path = Self::board_cache_key(board_id);

        if !invalidate_cache {
            if let Ok(data) = std::fs::read(&cache_path) {
                let parsed: Result<Vec<PostInBoard>, Vec<GlowficError>> =
                    serde_json::from_slice(&data).unwrap();
                if let Ok(posts) = parsed {
                    return Ok(Ok(posts));
                }
            }
        }

        let response = Self::board_get_all(board_id).await?;

        std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        write_if_changed(&cache_path, serde_json::to_vec_pretty(&response).unwrap()).unwrap();

        Ok(response)
    }
}

impl Threads {
    pub async fn board_get_all_cached(
        board_id: u64,
        invalidate_cache: bool,
    ) -> Result<Result<Vec<Thread>, Vec<GlowficError>>, Box<dyn Error>> {
        let mut threads = vec![];
        let board_posts = match BoardPosts::board_get_all_cached(board_id, invalidate_cache).await? {
            Ok(board_posts) => board_posts,
            Err(errors) => return Ok(Err(errors)),
        };

        for board_post in board_posts {
            let post = match Post::get_cached(board_post.id, invalidate_cache).await? {
                Ok(post) => post,
                Err(errors) => return Ok(Err(errors)),
            };
            match Replies::get_all_cached(post.id, invalidate_cache).await? {
                Ok(replies) => threads.push(Thread { post, replies }),
                Err(errors) => return Ok(Err(errors)),
            };
        }

        Ok(Ok(threads))
    }
    pub async fn cache_all_icons(&self, invalidate_cache: bool) {
        let icons: BTreeSet<_> = self.0
            .iter()
            .flat_map(|thread| thread.icons())
            .cloned()
            .collect();

        for icon in icons {
            if let Err(e) = icon.retrieve_cached(invalidate_cache).await {
                log::info!("{e:?}");
            }
        }
    }
}

impl SectionedThreads {
    pub async fn get_all_cached(
        board_id: u64,
        invalidate_cache: bool,
    ) -> Result<Result<Vec<(Section, Vec<Thread>)>, Vec<GlowficError>>, Box<dyn Error>> {
        let mut sections = match Board::get(board_id).await? {
            Ok(board) => board.board_sections,
            Err(errors) => return Ok(Err(errors)),
        };
        sections.push(Section::null());

        let mut sectioned_threads = vec![];
        for section in sections {
            sectioned_threads.push((section, vec![]));
        }
        sectioned_threads.sort_by(|a, b| a.0.order.cmp(&b.0.order));

        let threads = match Threads::board_get_all_cached(board_id, invalidate_cache).await? {
            Ok(threads) => threads,
            Err(errors) => return Ok(Err(errors)),
        };
        
        for thread in threads {
            let section_id = match thread.post.section {
                Some(ref section) => section.id,
                None => 0,
            };
            match sectioned_threads.iter_mut().find(|r| r.0.id == section_id) {
                Some(thread_section) => thread_section.1.push(thread),
                None => return Err(Box::<dyn Error>::from("Section {section_id} not found.")),
            };
        }

        for thread_section in &mut sectioned_threads {
            thread_section.1.sort_by(|a, b| a.post.section_order.cmp(&b.post.section_order));
        }

        return Ok(Ok(sectioned_threads));
    }
}

async fn get_cached_glowfic<T>(
    url: &str,
    cache_path: &Path,
    invalidate_cache: bool,
) -> Result<Result<T, Vec<GlowficError>>, Box<dyn Error>>
where
    T: DeserializeOwned + Serialize,
{
    if !invalidate_cache {
        if let Ok(data) = std::fs::read(cache_path) {
            let parsed: Result<T, Vec<GlowficError>> = serde_json::from_slice(&data).unwrap();
            if parsed.is_ok() {
                return Ok(parsed);
            }
        }
    }
    let response = crate::api::get_glowfic(url).await?;

    std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    write_if_changed(cache_path, serde_json::to_vec_pretty(&response).unwrap()).unwrap();

    Ok(response)
}

/// Avoids updating the last-modified date of the file.
pub fn write_if_changed(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    match std::fs::read(path.as_ref()) {
        Ok(data) if data == contents.as_ref() => Ok(()),
        Ok(_) | Err(_) => std::fs::write(path, contents),
    }
}
