use crate::model::{Font as LottieFont, Model};
use crate::Error;

#[cfg(feature = "fontkit")]
use alloc::string::String;
#[cfg(feature = "fontkit")]
use alloc::vec::Vec;
#[cfg(feature = "fontkit")]
use core::ops::Deref;
#[cfg(feature = "fontkit")]
use hashbrown::HashMap;

#[cfg(feature = "fontkit")]
use fontkit::{Font, FontKey, FontKit};

#[cfg(feature = "fontkit")]
const BUFFER_LIMIT: usize = 10 * 1_024 * 1_024;

#[cfg(feature = "fontkit")]
pub struct FontDB {
    fontkit: FontKit,
    font_map: HashMap<String, Vec<FontKey>>,
}

#[cfg(not(feature = "fontkit"))]
pub struct FontDB;

impl FontDB {
    pub fn new() -> Self {
        #[cfg(feature = "fontkit")]
        {
            FontDB {
                fontkit: FontKit::new(),
                font_map: HashMap::new(),
            }
        }
        #[cfg(not(feature = "fontkit"))]
        {
            FontDB
        }
    }

    pub fn load_fonts_from_model(&mut self, model: &Model) -> Result<(), Error> {
        #[cfg(feature = "fontkit")]
        {
            use alloc::vec;
            use std::io::Read;

            if let Some(font_dir) = dirs::font_dir() {
                self.fontkit
                    .search_fonts_from_path(font_dir.to_string_lossy().as_ref())?;
            }

            #[cfg(target_os = "macos")]
            self.fontkit
                .search_fonts_from_path("/System/Library/Fonts")?;

            for font in &model.fonts.list {
                if let Some(path) = font.path.as_ref() {
                    if font.origin == crate::model::FontPathOrigin::FontUrl {
                        let response = ureq::get(path).call()?;
                        let mut bytes = vec![];
                        response
                            .into_reader()
                            .take((BUFFER_LIMIT + 1) as u64)
                            .read_to_end(&mut bytes)?;
                        let keys = self.fontkit.add_font_from_buffer(bytes)?;
                        self.font_map.insert(font.name.clone(), keys);
                    }
                }
            }
        }
        let _ = model;
        Ok(())
    }

    #[cfg(feature = "fontkit")]
    pub fn font(&self, font: &LottieFont) -> Option<impl Deref<Target = Font> + '_> {
        match font.origin {
            crate::model::FontPathOrigin::Local
            | crate::model::FontPathOrigin::ScriptUrl
            | crate::model::FontPathOrigin::CssUrl => self
                .fontkit
                .query(&FontKey::new_with_family(font.name.clone()))
                .or_else(|| {
                    self.fontkit
                        .query(&FontKey::new_with_family(font.family.clone()))
                })
                .or_else(|| {
                    self.fontkit
                        .query(&FontKey::new_with_family("Fira Mono".to_string()))
                }),
            crate::model::FontPathOrigin::FontUrl => {
                self.fontkit.query(self.font_map.get(&font.name)?.first()?)
            }
        }
    }

    #[cfg(not(feature = "fontkit"))]
    pub fn font(&self, _font: &LottieFont) -> Option<()> {
        None
    }

    #[cfg(feature = "fontkit")]
    pub fn fontkit(&self) -> &FontKit {
        &self.fontkit
    }
}
