use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct Media {
    pub content: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Media {
    pub fn from_bytes(content: Vec<u8>, width: u32, height: u32) -> Self {
        Media {
            content,
            width,
            height,
        }
    }
}
