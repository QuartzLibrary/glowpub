use std::{
    collections::BTreeSet,
    error::Error,
    path::{Path, PathBuf},
    str::FromStr,
};

use mime::Mime;
use reqwest::header::CONTENT_TYPE;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    api::{BoardPosts, GlowficError, PostInBoard, Replies},
    types::{Continuity, Icon, Thread},
    utils::{
        extension_to_image_mime, guess_image_mime, http_client, mime_to_image_extension, url_hash,
    },
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

impl BoardPosts {
    fn cache_key(id: u64) -> PathBuf {
        format!("{CACHE_ROOT}/boards/{id}/posts.json").into()
    }

    pub async fn get_all_cached(
        id: u64,
        invalidate_cache: bool,
    ) -> Result<Result<Vec<PostInBoard>, Vec<GlowficError>>, Box<dyn Error>> {
        let cache_path = Self::cache_key(id);

        if !invalidate_cache {
            if let Ok(data) = std::fs::read(&cache_path) {
                let parsed: Result<Vec<PostInBoard>, Vec<GlowficError>> =
                    serde_json::from_slice(&data).unwrap();

                if let Ok(posts) = parsed {
                    return Ok(Ok(posts));
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

    pub async fn download_cached(
        &self,
        invalidate_cache: bool,
    ) -> Result<(Mime, Vec<u8>), Box<dyn Error>> {
        let Self { id, url, .. } = self;

        let Some(url) = url else {
            return Err("No url provided for this icon".into());
        };

        if !invalidate_cache {
            if let Ok((mime, data)) = read_image_file(Self::cache_key(*id, "*")) {
                return Ok((mime, data));
            }
        }

        log::info!("Downloading icon {id} from {url}");

        let (mime, data) = download_image(url).await?;

        let mime = guess_image_mime(&data).unwrap_or(mime);

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
    ) -> Result<Result<Self, Vec<GlowficError>>, Box<dyn Error>> {
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
            if let Err(e) = icon.download_cached(invalidate_cache).await {
                log::info!("{e:?}");
            }
        }
        for url in self.image_urls() {
            if let Err(e) = download_cached_image(&url, invalidate_cache).await {
                log::info!("{e:?}");
            }
        }
    }
}

impl Continuity {
    pub async fn get_cached(
        id: u64,
        invalidate_cache: bool,
    ) -> Result<Result<Self, Vec<GlowficError>>, Box<dyn Error>> {
        let board = match Board::get_cached(id, invalidate_cache).await? {
            Ok(board) => board,
            Err(errors) => return Ok(Err(errors)),
        };
        let threads = match BoardPosts::get_all_cached(id, invalidate_cache).await? {
            Ok(board_posts) => {
                let mut threads = vec![];
                for p in board_posts {
                    log::info!("Downloading post {} - {}", p.id, &p.subject);
                    let thread = match Thread::get_cached(p.id, invalidate_cache).await? {
                        Ok(thread) => thread,
                        Err(e) => return Ok(Err(e)),
                    };
                    threads.push(thread);
                }
                threads
            }
            Err(errors) => return Ok(Err(errors)),
        };

        Ok(Ok(Self { board, threads }))
    }
    pub async fn cache_all_icons(&self, invalidate_cache: bool) {
        let icons: BTreeSet<_> = self.threads.iter().flat_map(|t| t.icons()).collect();
        for icon in icons {
            if let Err(e) = icon.download_cached(invalidate_cache).await {
                log::info!("{e:?}");
            }
        }

        let urls: BTreeSet<_> = self.threads.iter().flat_map(|t| t.image_urls()).collect();
        for url in urls {
            if let Err(e) = download_cached_image(&url, invalidate_cache).await {
                log::info!("{e:?}");
            }
        }
    }
}

pub async fn download_cached_image(
    url: &str,
    invalidate_cache: bool,
) -> Result<(Mime, Vec<u8>), Box<dyn Error>> {
    fn image_cache_key(hash: &str, extension: &str) -> PathBuf {
        format!("{CACHE_ROOT}/images/hash_{hash}.{extension}").into()
    }

    let hash = url_hash(url);

    if !invalidate_cache {
        if let Ok((mime, data)) = read_image_file(image_cache_key(&hash, "*")) {
            return Ok((mime, data));
        }
    }

    log::info!("Downloading image {hash} from {url}");

    let (mime, data) = download_image(url).await?;

    let mime = guess_image_mime(&data).unwrap_or(mime);

    let extension = mime_to_image_extension(&mime).ok_or(format!("Invalid mime: {mime}"))?;

    let cache_path = image_cache_key(&hash, &extension);
    std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    write_if_changed(cache_path, &data).unwrap();

    Ok((mime, data))
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

pub async fn download_image(url: &str) -> Result<(Mime, Vec<u8>), reqwest::Error> {
    let response = http_client().get(url).send().await?;

    let content_type = response.headers().get(CONTENT_TYPE).unwrap();
    let mime = Mime::from_str(content_type.to_str().unwrap()).unwrap();

    let data = response.bytes().await?;

    Ok((mime, data.to_vec()))
}
fn read_image_file(path: PathBuf) -> Result<(Mime, Vec<u8>), Box<dyn Error>> {
    let files: Vec<_> = glob::glob(path.to_str().unwrap()).unwrap().collect();

    match &*files {
        // If we find a single file, we are good to go.
        [Ok(path)] => {
            let data = std::fs::read(path).unwrap();

            let extension = path.extension().unwrap().to_str().unwrap();
            if let Some(mime) = extension_to_image_mime(extension) {
                Ok((mime, data))
            } else {
                Err("Unsupprted extension in cached image.")?
            }
        }

        // The way we changed the handling of icons with broken mimes could lead to
        // multiple files for the same icon (but different extensions) being present.
        // We delete and re-download them.
        [_one, _two, _rest @ ..] => {
            #[allow(clippy::manual_flatten)] // Flattening [Result]s hides errors.
            for file in files {
                if let Ok(file) = file {
                    std::fs::remove_file(file).unwrap();
                }
            }

            Err(format!("Found multiple files for image ({path:?}). Cleaning them up. No further action needed."))?
        }

        _ => Err("Did not find a match for image in the cache.")?,
    }
}

/// Avoids updating the last-modified date of the file.
pub fn write_if_changed(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    match std::fs::read(path.as_ref()) {
        Ok(data) if data == contents.as_ref() => Ok(()),
        Ok(_) | Err(_) => std::fs::write(path, contents),
    }
}
