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
    /// Does not support collections. Only the first font in the file is used.
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

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.key, f)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           FontFamilyKey                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A font family key.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FontFamilyKey(u64);

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             FontFamily                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A font family.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct FontFamily {
    /// Family key.
    key: FontFamilyKey,
    /// Family name.
    name: String,
    /// Family variants.
    variants: HashMap<(FontWeight, FontStyle), FontKey>,
}

impl FontFamily {
    /// Creates a new family with `name`.
    pub fn new(key: FontFamilyKey, name: String) -> Self {
        Self {
            key,
            name,
            variants: Default::default(),
        }
    }

    /// Returns the key of this family.
    pub fn key(&self) -> FontFamilyKey {
        self.key
    }

    /// Returns the name of this family.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the variants in this family.
    pub fn variants(&self) -> &HashMap<(FontWeight, FontStyle), FontKey> {
        &self.variants
    }

    /// Returns the best match for `weight` and `style` in this family.
    pub fn best_match(&self, weight: FontWeight, style: FontStyle) -> Option<FontKey> {
        for &style in style.fallbacks() {
            for &weight in weight.fallbacks() {
                if let Some(key) = self.variants.get(&(weight, style)) {
                    return Some(*key);
                }
            }
        }

        None
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Fonts                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`Font`] collection.
///
/// Contains multiple [`Font`]s, organized in [`FontFamily`]s, and an emoji font fallback.
///
/// `FontFamilyKey`s ***must not*** be reused in other `Fonts`.
#[derive(Debug)]
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

    /// Returns the fonts map.
    pub fn fonts(&self) -> &HashMap<FontKey, Font> {
        &self.fonts
    }

    /// Returns the families map.
    pub fn families(&self) -> &HashMap<FontFamilyKey, FontFamily> {
        &self.families
    }

    /// Indexes in the collection.
    pub fn get<I: FontsGet>(&self, key: I) -> Option<I::Output<'_>> {
        key.get(self)
    }

    /// Inserts in the collection.
    pub fn set<I: FontsSet>(&mut self, key: I) -> Result<I::Output, I::Error> {
        key.set(self)
    }

    /// Returns a `FontRef` to the emoji font.
    pub fn emoji(&self) -> &Font {
        &self.emoji
    }

    /// Returns the next [`FontFamilyKey`] available for this `self.families`.
    fn family_key(&self) -> FontFamilyKey {
        let key = FontFamilyKey(self.families.len() as u64);

        // Family keys are not global.
        // `Fonts` neither, but should be constructed only once.
        debug_assert!(!self.families.contains_key(&key));

        key
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             FontsGet                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Overloads for [`Fonts::get`].
pub trait FontsGet {
    type Output<'fonts>;

    fn get<'fonts>(self, fonts: &'fonts Fonts) -> Option<Self::Output<'fonts>>;
}

impl FontsGet for FontKey {
    type Output<'fonts> = &'fonts Font;

    fn get<'fonts>(self, fonts: &'fonts Fonts) -> Option<Self::Output<'fonts>> {
        if self == fonts.emoji.key() {
            Some(&fonts.emoji)
        } else {
            fonts.fonts.get(&self)
        }
    }
}

impl FontsGet for FontFamilyKey {
    type Output<'fonts> = &'fonts FontFamily;

    fn get<'fonts>(self, fonts: &'fonts Fonts) -> Option<Self::Output<'fonts>> {
        fonts.families.get(&self)
    }
}

impl FontsGet for (FontFamilyKey, FontWeight, FontStyle) {
    type Output<'fonts> = &'fonts Font;

    fn get<'fonts>(self, fonts: &'fonts Fonts) -> Option<Self::Output<'fonts>> {
        fonts.get(fonts.get(self.0)?.best_match(self.1, self.2)?)
    }
}

impl FontsGet for &str {
    type Output<'fonts> = &'fonts FontFamily;

    fn get<'fonts>(self, fonts: &'fonts Fonts) -> Option<Self::Output<'fonts>> {
        fonts.families.values().find(|family| family.name == self)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             FontsSet                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Overloads for [`Fonts::set`].
pub trait FontsSet {
    type Output;
    type Error;

    fn set(self, fonts: &mut Fonts) -> Result<Self::Output, Self::Error>;
}

#[derive(Debug)]
pub enum InsertFontError {
    FontExists(Font),
}

impl FontsSet for Font {
    type Output = FontKey;
    type Error = InsertFontError;

    fn set(self, fonts: &mut Fonts) -> Result<Self::Output, Self::Error> {
        let key = self.key;

        if fonts.get(key).is_some() {
            Err(Self::Error::FontExists(self))
        } else {
            fonts.fonts.insert(key, self);
            Ok(key)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum InsertVariantError {
    FamilyNotFound,
    FontNotFound,
    VariantExists(FontKey),
}

impl FontsSet for (FontFamilyKey, FontWeight, FontStyle, FontKey) {
    type Output = (FontFamilyKey, FontKey);
    type Error = InsertVariantError;

    fn set(self, fonts: &mut Fonts) -> Result<Self::Output, Self::Error> {
        let (family_key, weight, style, font_key) = self;

        if fonts.get(font_key).is_none() {
            Err(Self::Error::FontNotFound)
        } else {
            if let Some(family) = fonts.families.get_mut(&family_key) {
                if let Some(variant) = family.variants.get(&(weight, style)) {
                    Err(Self::Error::VariantExists(*variant))
                } else {
                    family.variants.insert((weight, style), font_key);
                    Ok((family_key, font_key))
                }
            } else {
                Err(Self::Error::FamilyNotFound)
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CreateFamilyError {
    NameExists(FontFamilyKey),
}

impl FontsSet for String {
    type Output = FontFamilyKey;
    type Error = CreateFamilyError;

    fn set(self, fonts: &mut Fonts) -> Result<Self::Output, Self::Error> {
        if let Some(family) = fonts.get(self.as_str()) {
            Err(Self::Error::NameExists(family.key()))
        } else {
            let key = fonts.family_key();
            fonts.families.insert(key, FontFamily::new(key, self));
            Ok(key)
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Tests                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonts() {
        use CreateFamilyError::*;
        use FontStyle::*;
        use FontWeight::*;
        use InsertFontError::*;
        use InsertVariantError::*;

        fn key() -> FontKey {
            FontKey::new()
        }

        fn font(key: FontKey) -> Font {
            Font { data: vec![], key }
        }

        let emoji = key();

        // Create fonts
        let mut fonts = Fonts::new(font(emoji));

        // Emoji font exists
        assert!(fonts.get(emoji).unwrap().key() == emoji);

        // Insert font
        let regular = fonts.set(font(key())).unwrap();

        // Font exists
        assert!(fonts.get(regular).unwrap().key() == regular);

        // Re-insert font fails
        let err = fonts.set(font(regular)).unwrap_err();
        assert!(matches!(err, FontExists(font) if font.key() == regular));

        // Create family
        let family = fonts.set("Family".to_owned()).unwrap();

        // Family exists
        assert!(fonts.get(family).unwrap().key() == family);
        assert!(fonts.get("Family").unwrap().key() == family);

        // Re-create family fails
        assert!(fonts.set("Family".to_owned()) == Err(NameExists(family)));

        // Insert unknown font in family fails
        assert!(fonts.set((family, Regular, Normal, key())) == Err(FontNotFound));

        // Insert variant
        assert!(fonts.set((family, Regular, Normal, regular)) == Ok((family, regular)));

        // Re-insert variant fails
        assert!(fonts.set((family, Regular, Normal, regular)) == Err(VariantExists(regular)));

        // Variant exists
        assert!(fonts.get((family, Regular, Normal)).unwrap().key() == regular);

        // Variant falls back
        assert!(fonts.get((family, Regular, Italic)).unwrap().key() == regular);
    }
}
