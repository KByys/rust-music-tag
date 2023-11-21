use std::{
    fs::File,
    path::{Path, PathBuf},
};

use base64::Engine;
use id3::TagLike;
use symphonia::core::{formats::FormatReader, io::MediaSourceStream, probe::Hint};

use crate::{lyrics::Lyrics, AudioReader};

pub(crate) fn split_artists_to_string<'a>(iter: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut artists = Vec::new();
    for artist in iter {
        artists.append(&mut split_artist_to_string(artist))
    }
    artists
}
pub(crate) fn split_artist_to_string(name: &str) -> Vec<String> {
    let mut artists = Vec::new();
    for artist in name.split('/') {
        let mut tmp = artist
            .split('&')
            .filter_map(|f| {
                let t = f.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.into())
                }
            })
            .collect();
        artists.append(&mut tmp)
    }
    artists
}
use super::{Artwork, ImgFmt};
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MusicFormat {
    M4a,
    Mp3,
    Flac,
    Ogg,
}
#[derive(Debug)]
pub struct MusicTag {
    path: Option<PathBuf>,
    fmt: MusicFormat,
    title: Option<String>,
    artists: Vec<String>,
    album: Option<String>,
    album_artists: Vec<String>,
    year: Option<i32>,
    artworks: Vec<Artwork>,
    lyrics: Option<String>,
}

fn default_info(fmt: MusicFormat) -> MusicTag {
    MusicTag {
        path: None,
        fmt,
        title: None,
        artists: Vec::new(),
        album: None,
        album_artists: Vec::new(),
        year: None,
        artworks: Vec::new(),
        lyrics: None,
    }
}

impl TryFrom<(id3::Tag, Option<PathBuf>)> for MusicTag {
    type Error = crate::Error;

    fn try_from(value: (id3::Tag, Option<PathBuf>)) -> Result<Self, Self::Error> {
        let (tag, path) = value;
        let mut artists = Vec::new();
        for item in tag.artists().unwrap_or_default() {
            artists.append(&mut split_artist_to_string(item));
        }

        let album_artists = tag
            .album_artist()
            .map_or(Default::default(), split_artist_to_string);
        let mut artworks = Vec::new();
        for pic in tag.pictures() {
            if let id3::frame::PictureType::CoverFront = pic.picture_type {
                if let Ok(size) = imagesize::blob_size(&pic.data) {
                    artworks.push(Artwork {
                        data: pic.data.to_vec(),
                        height: size.height,
                        width: size.width,
                        fmt: ImgFmt::from_mime(&pic.mime_type),
                    })
                }
            }
        }
        let lyrics = tag.lyrics().next().map(|s| s.text.to_owned());
        Ok(MusicTag {
            path,
            fmt: MusicFormat::Mp3,
            title: tag.title().map(|s| s.to_string()),
            artists,
            album: tag.album().map(|s| s.to_string()),
            album_artists,
            year: tag.year(),
            artworks,
            lyrics,
        })
    }
}

impl TryFrom<(metaflac::Tag, Option<PathBuf>)> for MusicTag {
    type Error = crate::Error;

    fn try_from(value: (metaflac::Tag, Option<PathBuf>)) -> Result<Self, Self::Error> {
        fn get(key: &str, tag: &metaflac::Tag) -> Option<String> {
            tag.get_vorbis(key)
                .and_then(|mut a| a.next())
                .map(|s| s.into())
        }
        let (tag, path) = value;
        let mut artists = Vec::new();
        if let Some(artist_iter) = tag.get_vorbis("ARTIST") {
            for artist in artist_iter {
                artists.append(&mut split_artist_to_string(artist))
            }
        }
        let mut album_artists = Vec::new();
        if let Some(iter) = tag.get_vorbis("ALBUMARTIST") {
            for name in iter {
                album_artists.append(&mut split_artist_to_string(name))
            }
        };
        let mut artworks = Vec::new();
        for pic in tag.pictures() {
            if let metaflac::block::PictureType::CoverFront = pic.picture_type {
                if let Ok(size) = imagesize::blob_size(&pic.data) {
                    artworks.push(Artwork {
                        data: pic.data.to_vec(),
                        height: size.height,
                        width: size.width,
                        fmt: ImgFmt::from_mime(&pic.mime_type),
                    })
                }
            }
        }
        Ok(MusicTag {
            path,
            fmt: MusicFormat::Flac,
            title: get("TITLE", &tag),
            artists,
            album: get("ALBUM", &tag),
            album_artists,
            year: get("DATE", &tag).and_then(|year| year.parse().ok()),
            artworks,
            lyrics: get("LYRICS", &tag),
        })
    }
}

impl TryFrom<(mp4ameta::Tag, Option<PathBuf>)> for MusicTag {
    type Error = crate::Error;

    fn try_from(value: (mp4ameta::Tag, Option<PathBuf>)) -> Result<Self, Self::Error> {
        let (tag, path) = value;
        let artists = split_artists_to_string(tag.artists());
        let album_artists = split_artists_to_string(tag.album_artists());
        let mut artworks = Vec::new();
        for img in tag.artworks() {
            let fmt = if let mp4ameta::ImgFmt::Png = img.fmt {
                ImgFmt::PNG
            } else {
                ImgFmt::JPEG
            };
            if let Ok(size) = imagesize::blob_size(img.data) {
                artworks.push(Artwork {
                    height: size.height,
                    width: size.width,
                    data: img.data.to_owned(),
                    fmt,
                })
            }
        }
        let lyrics = tag.lyrics().map(|s| s.into());
        Ok(MusicTag {
            path,
            fmt: MusicFormat::M4a,
            title: tag.title().map(|s| s.to_string()),
            artists,
            album: tag.album().map(|s| s.to_string()),
            album_artists,
            year: tag.year().and_then(|year| year.parse().ok()),
            artworks,
            lyrics,
        })
    }
}

impl TryFrom<(Box<dyn FormatReader>, Option<PathBuf>)> for MusicTag {
    type Error = crate::Error;

    fn try_from(value: (Box<dyn FormatReader>, Option<PathBuf>)) -> Result<Self, Self::Error> {
        use symphonia::core::meta::Value;
        let mut info = default_info(MusicFormat::Ogg);
        let (mut reader, path) = value;
        info.path = path;
        if let Some(current) = reader.metadata().current() {
            for tag in current.tags() {
                match &tag.value {
                    Value::String(value) => match tag.key.as_str() {
                        "TITLE" => info.title = Some(value.into()),
                        "ALBUM" => info.album = Some(value.into()),
                        "ALBUMARTIST" => info
                            .album_artists
                            .append(&mut split_artist_to_string(value)),
                        "ARTIST" => info.artists.append(&mut split_artist_to_string(value)),
                        "DATE" => info.year = value.parse().ok(),
                        "LYRICS" => info.lyrics = Some(value.into()),
                        "METADATA_BLOCK_PICTURE" => {
                            if let Ok(pic) = base64::prelude::BASE64_STANDARD.decode(value) {
                                if let Ok(size) = imagesize::blob_size(&pic[42..]) {
                                    let image = Artwork {
                                        fmt: ImgFmt::JPEG,
                                        data: pic[42..].to_owned(),
                                        width: size.width,
                                        height: size.height,
                                    };
                                    info.artworks.push(image);
                                } else if let Ok(size) = imagesize::blob_size(&pic[41..]) {
                                    let image = Artwork {
                                        fmt: ImgFmt::JPEG,
                                        data: pic[41..].to_owned(),
                                        width: size.width,
                                        height: size.height,
                                    };
                                    info.artworks.push(image);
                                }
                            }
                        }
                        _ => (),
                    },
                    _ => continue,
                }
            }
        }
        Ok(info)
    }
}
impl MusicTag {
    pub fn read_from_bytes(bytes: impl Into<Vec<u8>>, fmt: MusicFormat) -> crate::Result<Self> {
        let reader = AudioReader::new(bytes.into());
        match fmt {
            MusicFormat::Mp3 => {
                use id3::Tag;
                let tag = Tag::read_from(reader)?;
                Self::try_from((tag, None))
            }
            MusicFormat::Flac => todo!(),
            _ => todo!(),
        }
    }
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let ext = match path.extension().and_then(|f| f.to_str()) {
            Some(ext) => ext,
            _ => return Err(crate::error::Error::FmtError("Not Supported".into())),
        };
        match ext {
            "mp3" => {
                use id3::Tag;

                let tag = Tag::read_from_path(&path)?;
                Self::try_from((tag, Some(path)))
            }
            "flac" => {
                use metaflac::Tag;
                let tag = Tag::read_from_path(&path)?;
                Self::try_from((tag, Some(path)))
            }
            "m4a" => {
                use mp4ameta::Tag;
                let tag = Tag::read_from_path(&path)?;
                Self::try_from((tag, Some(path)))
            }
            "ogg" => {
                let src = File::open(&path)?;
                let mss = MediaSourceStream::new(Box::new(src), Default::default());
                let mut hint = Hint::new();
                hint.with_extension("ogg");
                let probed = symphonia::default::get_probe().format(
                    &hint,
                    mss,
                    &Default::default(),
                    &Default::default(),
                )?;
                let format = probed.format;
                Self::try_from((format, Some(path)))
            }
            e => Err(crate::Error::FmtError(e.into())),
        }
    }

    pub fn as_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    pub fn fmt(&self) -> MusicFormat {
        self.fmt
    }
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }
    pub fn artist(&self) -> Option<&str> {
        self.artists.first().map(|s| s.as_str())
    }

    pub fn artists(&self) -> impl Iterator<Item = &str> {
        self.artists.iter().map(|s| s.as_str())
    }
    pub fn set_artists(&mut self, artists: Vec<impl Into<String>>) {
        self.artists = artists.into_iter().map(|s| s.into()).collect();
    }
    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }
    pub fn set_album(&mut self, album: &str) {
        self.album = Some(album.into())
    }
    pub fn album_artist(&self) -> Option<&str> {
        self.album_artists.first().map(|s| s.as_str())
    }
    pub fn album_artists(&self) -> impl Iterator<Item = &str> {
        self.album_artists.iter().map(|s| s.as_str())
    }
    pub fn set_album_artists(&mut self, album_artists: Vec<impl Into<String>>) {
        self.artists = album_artists.into_iter().map(|s| s.into()).collect();
    }
    pub fn year(&self) -> Option<i32> {
        self.year
    }
    pub fn set_year(&mut self, year: i32) {
        self.year = Some(year);
    }
    pub fn lyrics(&self) -> Option<Lyrics> {
        self.lyrics.as_ref().map(Lyrics::from)
    }
    pub fn artwork(&self) -> Option<&Artwork> {
        self.artworks.first()
    }
    pub fn artworks(&self) -> impl Iterator<Item = &Artwork> {
        self.artworks.iter()
    }
    pub fn add_artwork(&mut self, artwork: Artwork) {
        self.artworks.push(artwork)
    }
    pub fn set_artworks(
        &mut self,
        artworks: Vec<(impl Into<Vec<u8>>, ImgFmt)>,
    ) -> crate::Result<()> {
        let mut pics = Vec::new();
        for (data, fmt) in artworks {
            let data = data.into();
            let size = imagesize::blob_size(&data)?;
            pics.push(Artwork {
                height: size.height,
                width: size.width,
                data,
                fmt,
            });
        }
        self.artworks = pics;
        Ok(())
    }

    pub fn write_to_path(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        match self.fmt {
            MusicFormat::Mp3 => write_to_path_mp3(self, path),
            MusicFormat::Flac => write_to_path_flac(self, path),
            MusicFormat::M4a => write_to_path_m4a(self, path),
            _ => Err(crate::Error::NotSupportedError),
        }
    }
}

fn generate_artist(artists: &[String]) -> String {
    artists.iter().map(|s| format!("{}/", s)).collect()
}

fn write_to_path_m4a(info: &MusicTag, path: impl AsRef<Path>) -> crate::Result<()> {
    use mp4ameta::Tag;
    let mut tag = Tag::read_from_path(path.as_ref())?;

    tag.remove_title();
    if let Some(title) = info.title() {
        tag.set_title(title)
    };
    tag.remove_album();
    if let Some(album) = info.album() {
        tag.set_album(album)
    }
    tag.remove_year();
    if let Some(year) = info.year() {
        tag.set_year(year.to_string());
    }
    tag.set_artists(info.artists().map(|s| s.to_string()));
    if info.artists.is_empty() {
        tag.remove_artists();
    }
    tag.remove_album_artists();
    if !info.album_artists.is_empty() {
        tag.set_album_artists(info.album_artists().map(|s| s.to_string()))
    }
    use mp4ameta::Img;
    tag.remove_artworks();
    for artwork in info.artworks() {
        let fmt = match artwork.fmt {
            ImgFmt::JPEG => mp4ameta::ImgFmt::Jpeg,
            ImgFmt::PNG => mp4ameta::ImgFmt::Png,
        };
        tag.add_artwork(Img {
            fmt,
            data: artwork.data.clone(),
        })
    }
    tag.write_to_path(path)?;
    Ok(())
}

fn write_to_path_flac(info: &MusicTag, path: impl AsRef<Path>) -> crate::Result<()> {
    use metaflac::Tag;
    let mut tag = Tag::read_from_path(path.as_ref())?;
    fn set(key: &str, value: Option<impl Into<String>>, tag: &mut Tag) {
        tag.remove_vorbis(key);
        if let Some(value) = value {
            tag.set_vorbis(key, vec![value])
        }
    }
    set("TITLE", info.title(), &mut tag);
    set("ALBUM", info.album(), &mut tag);
    set("LYRICS", info.lyrics.as_deref(), &mut tag);
    let artists: Option<String> = if info.artists.is_empty() {
        Some(info.artists().map(|s| format!("{}/", s)).collect())
    } else {
        None
    };
    let album_artists: Option<String> = if info.album_artists.is_empty() {
        Some(info.artists().map(|s| format!("{}/", s)).collect())
    } else {
        None
    };
    set("ARTIST", artists, &mut tag);
    set("ALBUMARTIST", album_artists, &mut tag);
    set("DATE", info.year.map(|year| year.to_string()), &mut tag);
    use metaflac::block::{Picture, PictureType};
    use metaflac::Block;
    tag.remove_picture_type(PictureType::CoverFront);
    for artwork in info.artworks() {
        let pic = Picture {
            picture_type: PictureType::CoverFront,
            mime_type: artwork.mime_type().into(),
            data: artwork.data.clone(),
            ..Default::default()
        };
        tag.push_block(Block::Picture(pic));
    }
    tag.write_to_path(path)?;
    Ok(())
}

fn write_to_path_mp3(info: &MusicTag, path: impl AsRef<Path>) -> crate::Result<()> {
    use id3::Tag;
    let mut tag = Tag::read_from_path(path.as_ref())?;
    if let Some(title) = info.title() {
        tag.set_title(title)
    } else {
        tag.remove_title()
    };
    if let Some(album) = info.album() {
        tag.set_album(album)
    } else {
        tag.remove_album()
    }
    if let Some(year) = info.year() {
        tag.set_year(year)
    } else {
        tag.remove_year()
    }
    if info.artists.is_empty() {
        tag.remove_artist();
    } else {
        tag.set_artist(generate_artist(&info.artists))
    }
    if info.album_artists.is_empty() {
        tag.remove_album_artist();
    } else {
        tag.set_album_artist(generate_artist(&info.album_artists))
    }
    use id3::frame::{Lyrics, Picture, PictureType};
    tag.remove_all_lyrics();
    if let Some(lyrics) = &info.lyrics {
        tag.add_frame(Lyrics {
            lang: "utf-8".into(),
            description: String::new(),
            text: lyrics.into(),
        });
    }
    tag.remove_picture_by_type(PictureType::CoverFront);
    for artwork in info.artworks() {
        tag.add_frame(Picture {
            mime_type: artwork.mime_type().into(),
            picture_type: PictureType::CoverFront,
            description: Default::default(),
            data: artwork.data.clone(),
        });
    }
    tag.write_to_path(path, tag.version())?;
    Ok(())
}
