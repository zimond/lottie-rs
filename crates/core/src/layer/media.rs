use crate::model::Media as LottieMedia;
use base64::engine::general_purpose;
use base64::Engine as _;
use std::io::Read;
use std::path::PathBuf;
use url::Url;

use crate::Error;

#[derive(Debug, Clone)]
pub struct Media {
    pub content: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Media {
    pub fn new(media: LottieMedia, host: Option<&str>) -> Result<Self, Error> {
        // NOTE: by design `embedded` should have control over whether the image file is
        // base64 or not. But many lottie files simply do not take care so we
        // ignore it here.
        let mut url = match Url::parse(&media.pwd) {
            Ok(url) => url.join(&media.filename),
            Err(_) => Url::parse(&media.filename),
        }?;
        let content = if url.scheme() == "data" {
            let content = url.path().splitn(2, ",").nth(1).unwrap_or("");
            general_purpose::STANDARD.decode(content)?
        } else {
            let mut path = PathBuf::from(url.as_str());
            if !path.exists() {
                path = PathBuf::from(host.unwrap_or("")).join(path);
            }

            // For non-wasm32 target, try to load the file locally
            if path.exists() {
                let mut file = std::fs::File::open(path)?;
                let mut result = vec![];
                file.read_to_end(&mut result)?;
                result
            } else {
                if !url.has_host() {
                    url.set_host(host)?;
                }
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
