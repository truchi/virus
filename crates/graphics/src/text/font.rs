use std::path::Path;
use swash::{CacheKey, FontDataRef, FontRef};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Font                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A font.
pub struct Font {
    /// Font data.
    data: Vec<u8>,
    /// Font key.
    key: CacheKey,
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
    pub fn key(&self) -> CacheKey {
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
