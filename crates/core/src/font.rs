use fontkit::{Font, FontKey, FontKit, Width};
use lottie_model::{Font as LottieFont, FontPathOrigin};

pub(crate) trait FontLoader {
    fn fetch_font(&self, font: &LottieFont) -> Option<&Font>;
}

impl FontLoader for FontKit {
    fn fetch_font(&self, font: &LottieFont) -> Option<&Font> {
        match font.origin {
            FontPathOrigin::Local => {
                self.query(&FontKey::new(&font.family, 400, false, Width::from(5)))
            }
            _ => todo!(),
        }
    }
}
