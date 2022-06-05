use fontkit::FontKit;
use lottie_model::{Model, ShapeGroup, TextAnimationData};

use crate::prelude::RenderableContent;
use crate::Error;

impl RenderableContent {
    pub fn from_text(
        text: &TextAnimationData,
        model: &Model,
        fontkit: &FontKit,
    ) -> Result<Self, Error> {
        let mut frames = vec![];
        for keyframe in &text.data.keyframes {
            let doc = &keyframe.start_value;
            let text = model
                .font(&doc.font_family)
                .ok_or_else(|| Error::FontFamilyNotFound(doc.font_family.clone()))?;
            frames.push(());
        }
        Ok(RenderableContent::Shape(ShapeGroup { shapes: vec![] }))
    }
}
