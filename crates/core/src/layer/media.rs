use lottie_model::Media as LottieMedia;
use std::io::Read;
use url::{ParseError, Url};

use crate::Error;

#[derive(Debug, Clone)]
pub struct Media {
    pub content: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Media {
    pub fn new(media: LottieMedia, host: Option<String>) -> Result<Self, Error> {
        // NOTE: by design `embedded` should have control over whether the image file is
        // base64 or not. But many lottie files simply do not take care so we
        // ignore it here.
        let content = if media.filename.starts_with("data:") {
            let content = media.filename.splitn(2, ",").nth(1).unwrap_or("");
            base64::decode(content)?
        } else {
            let path = media.path();
            // For non-wasm32 target, try to load the file locally
            if path.exists() {
                let mut file = std::fs::File::open(path)?;
                let mut result = vec![];
                file.read_to_end(&mut result)?;
                result
            } else {
                let path = path.as_os_str().to_str().unwrap_or("");
                let url = path.parse::<Url>();
                let no_host = url
                    .as_ref()
                    .map(|url| !url.has_host())
                    .unwrap_or_else(|e| *e == ParseError::RelativeUrlWithoutBase);
                let url = if no_host {
                    if let Some(host) = host {
                        let url = host.parse::<Url>()?;
                        url.join(&path)?
                    } else {
                        url?
                    }
                } else {
                    url?
                };
                let url = url.as_str();
                let response = ureq::get(url).call()?;
                let len: usize = response
                    .header("Content-Length")
                    .ok_or_else(|| Error::NetworkMissingContentLength(url.to_string()))?
                    .parse()
                    .map_err(|_| Error::NetworkMalformedContentLength(url.to_string()))?;

                let mut bytes: Vec<u8> = Vec::with_capacity(len);
                response
                    .into_reader()
                    .take(len as u64)
                    .read_to_end(&mut bytes)?;
                bytes
            }
        };
        Ok(Media {
            content,
            width: media.width.unwrap_or_default(),
            height: media.height.unwrap_or_default(),
        })
    }
}
