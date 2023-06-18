use image::imageops::FilterType;
use std::{collections::HashMap, fmt::Debug, path::PathBuf, time::Duration};
use swash::{
    scale::{
        image::{Content, Image},
        Source, StrikeWith,
    },
    zeno::Placement,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          AnimatedGlyphId                                       //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub type AnimatedGlyphId = u16;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           AnimatedGlyph                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct AnimatedGlyph {
    id: AnimatedGlyphId,
    path: PathBuf,
    frames: Vec<Duration>,
    duration: Duration,
}

impl AnimatedGlyph {
    pub fn id(&self) -> AnimatedGlyphId {
        self.id
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn frame(&self, time: Duration) -> usize {
        let time = time.as_millis() % self.duration.as_millis();

        self.frames
            .iter()
            .enumerate()
            .scan(Duration::ZERO, |duration, (frame, delay)| {
                *duration += *delay;
                Some((frame, duration.as_millis()))
            })
            .find(|(_, duration)| time < *duration)
            .unwrap()
            .0
    }

    pub fn render(&self, size: u32) -> std::io::Result<Vec<Image>> {
        let path = self.path.to_str().unwrap();

        self.frames
            .iter()
            .enumerate()
            .map(|(frame, delay)| {
                let delay = delay.as_millis();
                let image =
                    image::io::Reader::open(format!("{path}/frame:{frame}-delay:{delay}.png"))?
                        .decode()
                        .unwrap();
                debug_assert!(image.width() == image.height());

                Ok(Image {
                    source: Source::ColorBitmap(StrikeWith::ExactSize /* NOTE ? */),
                    content: Content::Color,
                    placement: Placement {
                        top: 0,
                        left: 0,
                        width: size,
                        height: size,
                    },
                    data: image
                        .resize(size, size, Self::FILTER)
                        .into_rgba8()
                        .into_vec(),
                })
            })
            .collect()
    }
}

/// Private.
impl AnimatedGlyph {
    const FILTER: FilterType = FilterType::CatmullRom;

    fn new(id: AnimatedGlyphId, path: PathBuf) -> std::io::Result<Self> {
        const EXTENSION: &str = ".png";

        let mut frames = Vec::new();
        let mut duration = Duration::ZERO;

        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;

            if !entry.metadata()?.is_file() {
                continue;
            }

            let name = entry.file_name().into_string().unwrap();

            if !name.ends_with(EXTENSION) {
                continue;
            }

            let split = &mut name[0..name.len() - EXTENSION.len()].split('-');
            let mut parse = || {
                split
                    .next()
                    .unwrap()
                    .split(':')
                    .nth(1)
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            };
            let (frame, delay) = (parse(), Duration::from_millis(parse()));

            frames.push((frame, delay));
            duration += delay;
        }

        Ok(Self {
            id,
            path,
            frames: {
                frames.sort_unstable_by_key(|frame| frame.0);
                frames.into_iter().map(|(_, duration)| duration).collect()
            },
            duration,
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           AnimatedFont                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Default, Debug)]
pub struct AnimatedFont {
    ids: HashMap<String, AnimatedGlyphId>,
    glyphs: Vec<AnimatedGlyph>,
}

impl AnimatedFont {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let mut glyphs = Vec::new();
        let mut ids = HashMap::new();

        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;

            if !entry.metadata()?.is_dir() {
                continue;
            }

            let id = glyphs.len() as AnimatedGlyphId;
            let string = entry
                .file_name()
                .into_string()
                .unwrap()
                .split('_')
                .map(|u| u32::from_str_radix(u, 16).unwrap())
                .map(|u32| char::from_u32(u32).unwrap())
                .collect::<String>();
            let glyph = AnimatedGlyph::new(id, entry.path())?;

            glyphs.push(glyph);
            ids.insert(string, id);
        }

        Ok(Self { glyphs, ids })
    }

    pub fn get_by_str(&self, str: &str) -> Option<&AnimatedGlyph> {
        self.ids.get(str).and_then(|id| self.get_by_id(*id))
    }

    pub fn get_by_id(&self, id: AnimatedGlyphId) -> Option<&AnimatedGlyph> {
        self.glyphs.get(id as usize)
    }
}
