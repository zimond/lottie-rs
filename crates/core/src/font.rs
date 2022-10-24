use std::collections::HashMap;
use std::io::Read;

use fontkit::{Font, FontKey, FontKit, Width};
use lottie_model::{Font as LottieFont, FontPathOrigin, Model};

use crate::Error;

pub struct FontDB {
    fontkit: FontKit,
    font_map: HashMap<String, FontKey>,
}

impl FontDB {
    pub fn new(fontkit: FontKit) -> Self {
        FontDB {
            fontkit,
            font_map: HashMap::new(),
        }
    }

    pub fn load_fonts_from_model(&mut self, model: &Model) -> Result<(), Error> {
        // load remote fonts
        for font in &model.fonts.list {
            if let Some(path) = font.path.as_ref() {
                if font.origin == FontPathOrigin::FontUrl {
                    let response = ureq::get(path).call()?;
                    let len: usize = response.header("Content-Length")
                        .ok_or_else(|| Error::NetworkMissingContentLength(path.clone()))?
                        .parse().map_err(|_| Error::NetworkMalformedContentLength(path.clone()))?;

                    let mut bytes: Vec<u8> = Vec::with_capacity(len);
                    response.into_reader()
                        .take(len as u64)
                        .read_to_end(&mut bytes)?;
                    let key = self.fontkit.add_font_from_buffer(bytes)?;
                    self.font_map.insert(font.name.clone(), key);
                }
            }
        }
        Ok(())
    }

    pub fn font(&self, font: &LottieFont) -> Option<&Font> {
        match font.origin {
            FontPathOrigin::Local => {
                self.fontkit
                    .query(&FontKey::new(&font.family, 400, false, Width::from(5)))
            }
            FontPathOrigin::FontUrl => self.fontkit.query(self.font_map.get(&font.name)?),
            _ => todo!(),
        }
    }
}
