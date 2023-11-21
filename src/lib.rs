pub mod lyrics;
pub mod audio;
pub mod error;
pub use error::*;

pub use imagesize;
use std::io::{ErrorKind, SeekFrom};
use symphonia::core::io::MediaSource;
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MusicFmt {
    M4a,
    Mp3,
    Flac,
    Ogg,
}

pub struct AudioReader {
    pos: usize,
    buf: Vec<u8>,
}
impl MediaSource for AudioReader {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.buf.len() as u64)
    }
}
unsafe impl Send for AudioReader {}
unsafe impl Sync for AudioReader {}
impl AudioReader {
    pub fn new(buf: Vec<u8>) -> AudioReader {
        Self { pos: 0, buf }
    }
}
impl std::io::Seek for AudioReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Current(i) => {
                let pos = i as usize + self.pos;
                if self.buf.len() > pos {
                    self.pos = pos;
                    Ok(self.pos as u64)
                } else {
                    Err(std::io::Error::new(
                        ErrorKind::InvalidInput,
                        "Invalid position",
                    ))
                }
            }
            SeekFrom::End(e) => {
                if self.buf.len() >= e as usize {
                    self.pos = self.buf.len() - e as usize;
                    Ok(self.pos as u64)
                } else {
                    Err(std::io::Error::new(
                        ErrorKind::InvalidInput,
                        "Invalid position",
                    ))
                }
            }
            SeekFrom::Start(s) => {
                if self.buf.len() >= s as usize {
                    self.pos = s as usize;
                    Ok(self.pos as u64)
                } else {
                    Err(std::io::Error::new(
                        ErrorKind::InvalidInput,
                        "Invalid position",
                    ))
                }
            }
        }
    }
}
impl std::io::Read for AudioReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let pos = self.pos;
        for i in self.pos..self.buf.len() {
            if i - pos >= buf.len() {
                break;
            }
            buf[i - pos] = self.buf[i];
            self.pos = i + 1;
        }
        Ok(self.pos - pos)
    }
}
