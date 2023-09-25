use std::collections::HashMap;
use std::io::Read;
use std::ops::Deref;

use fontkit::{Font, FontKey, FontKit};
use lottie_model::{Font as LottieFont, FontPathOrigin, Model};

use crate::Error;

pub struct FontDB {
    fontkit: FontKit,
    font_map: HashMap<String, Vec<FontKey>>,
}

impl FontDB {
    pub fn new(fontkit: FontKit) -> Self {
        FontDB {
            fontkit,
            font_map: HashMap::new(),
        }
    }

    pub fn load_fonts_from_model(&mut self, model: &Model) -> Result<(), Error> {
        // load default font
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut path = std::env::current_dir()?;
            path.push("assets/FiraMono-Regular.ttf");
            self.fontkit
                .search_fonts_from_path(&path.into_os_string().into_string().unwrap_or_default())?;
        }
        // load remote fonts
        for font in &model.fonts.list {
            if let Some(path) = font.path.as_ref() {
                if font.origin == FontPathOrigin::FontUrl {
                    let response = ureq::get(path).call()?;
                    let len: usize = response
                        .header("Content-Length")
                        .ok_or_else(|| Error::NetworkMissingContentLength(path.clone()))?
                        .parse()
                        .map_err(|_| Error::NetworkMalformedContentLength(path.clone()))?;

                    let mut bytes: Vec<u8> = Vec::with_capacity(len);
                    response
                        .into_reader()
                        .take(len as u64)
                        .read_to_end(&mut bytes)?;
                    let keys = self.fontkit.add_font_from_buffer(bytes)?;
                    self.font_map.insert(font.name.clone(), keys);
                }
            }
        }
        Ok(())
    }

    pub fn font(&self, font: &LottieFont) -> Option<impl Deref<Target = Font> + '_> {
        match font.origin {
            // This is not an html player. So we treat script/css urls as local obtained fonts
            // TODO: could this be a thing in WASM target?
            FontPathOrigin::Local | FontPathOrigin::ScriptUrl | FontPathOrigin::CssUrl => self
                .fontkit
                .query(&FontKey::new_with_family(font.name.clone()))
                .or_else(|| {
                    self.fontkit
                        .query(&FontKey::new_with_family(font.family.clone()))
                })
                .or_else(|| {
                    // default font
                    self.fontkit
                        .query(&FontKey::new_with_family("Fira Mono".to_string()))
                }),
            // TODO: What if font from url is *.ttc and font.name points to one font in the
            // collection? Could this be possible?
            FontPathOrigin::FontUrl => self.fontkit.query(self.font_map.get(&font.name)?.first()?),
        }
    }

    pub fn fontkit(&self) -> &FontKit {
        &self.fontkit
    }
}
