use std::path::{Path, PathBuf};

use anyhow::{Error, Result};
use config::PREVIEW;
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use md5::{Digest, Md5};
use shared::tty_ratio;
use tokio::fs;

pub struct Image;

impl Image {
    pub(super) async fn crop(path: &Path, size: (u16, u16)) -> Result<DynamicImage> {
        let (w, h) = {
            let r = tty_ratio();
            let (w, h) = ((size.0 as f64 * r.0) as u32, (size.1 as f64 * r.1) as u32);
            (w.min(PREVIEW.max_width), h.min(PREVIEW.max_height))
        };

        let img = fs::read(path).await?;
        let img = tokio::task::spawn_blocking(move || -> Result<DynamicImage> {
            let img = image::load_from_memory(&img)?;
            Ok(if img.width() > w || img.height() > h {
                img.resize(w, h, FilterType::Triangle)
            } else {
                img
            })
        });

        img.await?
    }

    pub async fn precache(img: Vec<u8>, cache: PathBuf) -> Result<()> {
        let result = tokio::task::spawn_blocking(move || {
            let img = image::load_from_memory(&img)?;
            let (w, h) = (PREVIEW.max_width, PREVIEW.max_height);

            if img.width() > w || img.height() > h {
                img.resize(w, h, FilterType::Triangle)
                    .save_with_format(cache, ImageFormat::Jpeg)?;
            }
            Ok::<(), Error>(())
        });

        result.await?
    }

    #[inline]
    pub fn cache(path: &Path) -> PathBuf {
        format!(
            "/tmp/yazi/{:x}",
            Md5::new_with_prefix(path.to_string_lossy().as_bytes()).finalize()
        )
        .into()
    }
}
