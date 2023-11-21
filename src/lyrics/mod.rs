#[derive(Debug, Default)]
pub struct Lyrics {
    lines: Vec<String>,
}

impl<T: AsRef<str>> From<T> for Lyrics {
    fn from(value: T) -> Self {
        Self {
            lines: split_lyrics(value.as_ref())
                .into_iter()
                .map(|s| s.to_owned())
                .collect(),
        }
    }
}

fn split_lyrics(lyrics: &str) -> Vec<String> {
    let mut pre_pos = 0;
    let mut pos = 0;
    let bytes = lyrics.as_bytes();
    let mut lyc = Vec::new();
    while pos < bytes.len() {
        let b = bytes[pos];
        if b == b'\r' || b == b'\n' {
            let str = lyrics[pre_pos..pos].trim().to_string();
            lyc.push(str);
            if pos + 1 < bytes.len() && (bytes[pos + 1] == b'\n' || bytes[pos + 1] == b'\r') {
                pre_pos = pos + 2;
            } else {
                pre_pos = pos + 1;
            }
            pos += 1;
        }
        pos += 1;
    }
    lyc
}

impl Lyrics {
    pub fn lines(&self) -> &Vec<String> {
        &self.lines
    }
    pub fn lines_with_time(&self) -> impl Iterator<Item = (Option<LyricsDuration>, &str)> {
        self.lines.iter().map(|s| get_duration(s))
    }
}

#[derive(Debug, Default)]
pub struct LyricsDuration {
    min: u64,
    secs: u64,
    milliseconds: u64,
}
impl std::fmt::Display for LyricsDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:0>2}:{:0>2}.{:0>2}",
            self.min, self.secs, self.milliseconds
        ))
    }
}

impl LyricsDuration {
    pub fn from_min_secs_f64(min: u64, secs: f64) -> LyricsDuration {
        let secs = (secs * 100.0) as u64;
        Self {
            min,
            secs: secs / 100 % 60,
            milliseconds: secs % 100,
        }
    }
    pub fn minute(&self) -> u64 {
        self.min
    }
    pub fn seconds(&self) -> u64 {
        self.secs
    }
    pub fn milliseconds(&self) -> u64 {
        self.milliseconds
    }
}

fn parse_time(time: &str) -> Option<LyricsDuration> {
    let min: u64 = time[0..2].parse().ok()?;
    let secs: f64 = time[3..].parse().ok()?;
    Some(LyricsDuration::from_min_secs_f64(min, secs))
}
fn get_duration(line: &str) -> (Option<LyricsDuration>, &str) {
    if let Some(time) = line.get(0..10) {
        if time.starts_with('[') && time.ends_with(']') {
            return (parse_time(&time[1..9]), &line[10..]);
        }
    }
    (None, line)
}

pub enum LyricsType {
    Lrc,
    Rlrc,
}
pub struct RichLyrics {}

pub struct WordSpace {}
pub struct Word {
    time: String,
    word: String,
}
pub enum LyricTag {
    Wait,
    P(),
    Lyricist(Vec<String>),
    Composer(Vec<String>),
}

pub struct RichLyricsLine {}

/*
    <lyricist></lyricist> 写词人
    <composer> 作曲家
    <p name="" hor="left or right" end="00:00.00">
        <word-space> or <word>
           <duration>00:00.00</>
           <w>Hello</w>
           </>
    </p>
    <wait start="00:00.34" > // 自动生成, 3s

*/
