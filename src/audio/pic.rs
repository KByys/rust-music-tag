use std::fmt::Debug;

#[derive(Debug)]
pub enum ImgFmt {
    JPEG,
    PNG,
}
impl ImgFmt {
    pub fn from_mime(mime: &str) -> Self {
        match mime {
            "image/png" => ImgFmt::PNG,
            _ => ImgFmt::JPEG,
        }
    }
}
pub struct Artwork {
    pub height: usize,
    pub width: usize,
    pub data: Vec<u8>,
    pub fmt: ImgFmt,
}
impl Artwork {
    pub fn mime_type(&self) -> &'static str {
        match self.fmt {
            ImgFmt::JPEG => "image/jpeg",
            ImgFmt::PNG => "image/png",
        }
    }
}
impl Debug for Artwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Artwork")
            .field("height", &self.height)
            .field("width", &self.width)
            .field("fmt", &self.fmt)
            .finish()
    }
}
