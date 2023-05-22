use std::{collections::HashMap, path::Path};
use swash::{CacheKey, FontDataRef, FontRef};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             FontWeight                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Font weight.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub enum FontWeight {
    /// 100.
    Thin,
    /// 200.
    ExtraLight,
    /// 300.
    Light,
    /// 400.
    #[default]
    Regular,
    /// 500.
    Medium,
    /// 600.
    SemiBold,
    /// 700.
    Bold,
    /// 800.
    ExtraBold,
    /// 900.
    Black,
}

impl FontWeight {
    pub fn fallbacks(&self) -> &'static [Self] {
        use FontWeight::*;

        match self {
            // Descending, then ascending
            Thin => &[
                Thin, ExtraLight, Light, Regular, Medium, SemiBold, Bold, ExtraBold, Black,
            ],
            ExtraLight => &[
                ExtraLight, Thin, Light, Regular, Medium, SemiBold, Bold, ExtraBold, Black,
            ],
            Light => &[
                Light, ExtraLight, Thin, Regular, Medium, SemiBold, Bold, ExtraBold, Black,
            ],

            // Up, then descending, then ascending
            Regular => &[
                Regular, Medium, Light, ExtraLight, Thin, SemiBold, Bold, ExtraBold, Black,
            ],

            // Down, then ascending, then descending
            Medium => &[
                Medium, Regular, SemiBold, Bold, ExtraBold, Black, Light, ExtraLight, Thin,
            ],

            // Ascending, then descending
            SemiBold => &[
                SemiBold, Bold, ExtraBold, Black, Medium, Regular, Light, ExtraLight, Thin,
            ],
            Bold => &[
                Bold, ExtraBold, Black, SemiBold, Medium, Regular, Light, ExtraLight, Thin,
            ],
            ExtraBold => &[
                ExtraBold, Black, Bold, SemiBold, Medium, Regular, Light, ExtraLight, Thin,
            ],
            Black => &[
                Black, ExtraBold, Bold, SemiBold, Medium, Regular, Light, ExtraLight, Thin,
            ],
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              FontStyle                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Font style.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub enum FontStyle {
    /// Normal.
    #[default]
    Normal,
    /// Italic.
    Italic,
    /// Oblique.
    Oblique,
}

impl FontStyle {
    pub fn fallbacks(&self) -> &'static [Self] {
        use FontStyle::*;

        match self {
            Normal => &[Normal, Oblique, Italic],
            Italic => &[Italic, Oblique, Normal],
            Oblique => &[Oblique, Italic, Normal],
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              FontKey                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A font key.
pub type FontKey = CacheKey;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Font                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A font.
pub struct Font {
    /// Font data.
    data: Vec<u8>,
    /// Font key.
    key: FontKey,
}

impl Font {
    /// Returns a `Font` from `path`.
    ///
    /// Does not support collections. Currently, only the first font in the file is used.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Option<Self> {
        let data = std::fs::read(path).ok()?;
        let font = FontRef::from_index(&data, 0)?;
        let key = font.key;

        debug_assert!(font.offset == 0);

        Some(Self { data, key })
    }

    /// Returns a `FontRef` from this `Font`.
    pub fn as_ref(&self) -> FontRef {
        FontRef {
            data: &self.data,
            offset: 0,
            key: self.key,
        }
    }

    /// Returns the key.
    pub fn key(&self) -> FontKey {
        self.key
    }

    /// Prints various infos from the [`Font`] file.
    pub fn inspect(&self) {
        let font = self.as_ref();

        println!(
            "--- Is collection? {} ---",
            FontDataRef::new(font.data)
                .map(|f| f.is_collection())
                .unwrap_or_default()
        );

        println!("--- Color palettes: {} ---", font.color_palettes().count());
        println!("--- Bitmap strikes: {} ---", font.alpha_strikes().count());
        println!("--- Color strikes: {} ---", font.color_strikes().count());

        println!("--- Attibutes ---");
        println!("    {}", font.attributes());

        println!("--- Features: {} ---", font.features().count());
        for feature in font.features() {
            println!(
                "    [{:?}] {:?} {:?}",
                feature.tag(),
                feature.name(),
                feature.action()
            );
        }

        println!("--- Instances: {} ---", font.instances().count());
        for instance in font.instances() {
            let name = instance
                .name(None)
                .map(|i| i.to_string())
                .unwrap_or_default();
            println!("    [{}] {name}", instance.index());
        }

        println!(
            "--- Writing systems: {} ---",
            font.writing_systems().count()
        );
        for system in font.writing_systems() {
            let script = system.script();
            let language = system.language().map(|l| l.to_string());
            println!("    {script:?} {language:?}");
        }

        let strings = font
            .localized_strings()
            .map(|string| (string.id().to_raw(), string))
            .collect::<std::collections::HashMap<_, _>>();
        let mut strings = strings.into_iter().collect::<Vec<_>>();
        strings.sort_by_key(|(id, _)| *id);

        println!("--- Localized strings: {} ---", strings.len());
        for (_, string) in strings {
            println!("    [{:?}] {}", string.id(), string.to_string());
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           FontFamilyKey                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A font family key.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FontFamilyKey(u64);

impl FontFamilyKey {
    /// For tests only!
    pub fn __new(u64: u64) -> Self {
        Self(u64)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             FontFamily                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A font family.
#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct FontFamily(HashMap<(FontWeight, FontStyle), FontKey>);

impl FontFamily {
    pub fn get(&self, weight: FontWeight, style: FontStyle) -> Option<FontKey> {
        for &style in style.fallbacks() {
            for &weight in weight.fallbacks() {
                if let Some(key) = self.0.get(&(weight, style)) {
                    return Some(*key);
                }
            }
        }

        None
    }

    pub fn insert(
        &mut self,
        weight: FontWeight,
        style: FontStyle,
        key: FontKey,
    ) -> Option<FontKey> {
        self.0.insert((weight, style), key)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Fonts                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`Font`] collection.
///
/// Contains multiple [`Font`]s, organized in [`FontFamily`], and an emoji font fallback.
pub struct Fonts {
    /// Fonts in the collection.
    fonts: HashMap<FontKey, Font>,
    /// Families in the collection.
    families: HashMap<FontFamilyKey, FontFamily>,
    /// The emoji fallback font.
    emoji: Font,
}

impl Fonts {
    /// Returns a new `Fonts` with `emoji` fallback.
    pub fn new(emoji: Font) -> Self {
        Self {
            fonts: Default::default(),
            families: Default::default(),
            emoji,
        }
    }

    pub fn fonts(&self) -> &HashMap<FontKey, Font> {
        &self.fonts
    }

    pub fn families(&self) -> &HashMap<FontFamilyKey, FontFamily> {
        &self.families
    }

    /// Returns a `FontRef` from a `key`.
    pub fn get<I: FontsGet>(&self, key: I) -> Option<FontRef> {
        key.get(self)
    }

    /// Returns a `FontRef` from a `key`.
    pub fn set<I: FontsSet>(&mut self, key: I) -> I::Output {
        key.set(self)
    }

    /// Returns a `FontRef` to the emoji font.
    pub fn emoji(&self) -> FontRef {
        self.emoji.as_ref()
    }

    /// Returns the next [`FontFamilyKey`].
    fn family_key(&self) -> FontFamilyKey {
        let key = FontFamilyKey(self.families.len() as u64);
        debug_assert!(self.families.contains_key(&key));
        key
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             FontsGet                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub trait FontsGet {
    fn get(self, fonts: &Fonts) -> Option<FontRef>;
}

impl FontsGet for FontKey {
    fn get(self, fonts: &Fonts) -> Option<FontRef> {
        if self == fonts.emoji.key() {
            Some(fonts.emoji.as_ref())
        } else {
            fonts.fonts.get(&self).map(|font| font.as_ref())
        }
    }
}

impl FontsGet for (FontFamilyKey, FontWeight, FontStyle) {
    fn get(self, fonts: &Fonts) -> Option<FontRef> {
        fonts.families.get(&self.0)?.get(self.1, self.2)?.get(fonts)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             FontsSet                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub trait FontsSet {
    type Output;

    fn set(self, fonts: &mut Fonts) -> Self::Output;
}

impl FontsSet for Font {
    type Output = FontKey;

    fn set(self, fonts: &mut Fonts) -> Self::Output {
        let key = self.key;

        debug_assert!(!fonts.fonts.contains_key(&key));
        fonts.fonts.insert(key, self);

        key
    }
}

impl FontsSet for (FontFamilyKey, FontWeight, FontStyle, FontKey) {
    type Output = FontFamilyKey;

    fn set(self, fonts: &mut Fonts) -> Self::Output {
        let (family_key, weight, style, font_key) = self;

        debug_assert!(fonts.fonts.contains_key(&font_key));
        fonts
            .families
            .entry(family_key)
            .or_default()
            .insert(weight, style, font_key);

        family_key
    }
}

impl FontsSet for (FontWeight, FontStyle, FontKey) {
    type Output = FontFamilyKey;

    fn set(self, fonts: &mut Fonts) -> Self::Output {
        fonts.set((fonts.family_key(), self.0, self.1, self.2))
    }
}

impl FontsSet for (FontFamilyKey, &[(FontWeight, FontStyle, FontKey)]) {
    type Output = FontFamilyKey;

    fn set(self, fonts: &mut Fonts) -> Self::Output {
        for &(weight, style, font_key) in self.1 {
            fonts.set((self.0, weight, style, font_key));
        }

        self.0
    }
}

impl FontsSet for &[(FontWeight, FontStyle, FontKey)] {
    type Output = FontFamilyKey;

    fn set(self, fonts: &mut Fonts) -> Self::Output {
        fonts.set((fonts.family_key(), self))
    }
}
