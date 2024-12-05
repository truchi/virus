pub mod panes;
pub mod syntax {
    mod capture;
    mod lines;
    mod theme;

    pub use capture::*;
    pub use lines::*;
    pub use theme::*;
}
pub mod theme;
pub mod tween;
pub mod ui;
pub mod views {
    mod document;
    mod files;

    pub use document::*;
    pub use files::*;
}

// For convenience.
pub use virus_graphics::Catppuccin;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              TODO                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

mod todo {
    use crate::{syntax::SyntaxTheme, theme::UiTheme, tween::Tween};
    use std::time::Duration;
    use virus_graphics::{
        text::{
            Context, Font, FontSize,
            FontStyle::{self, *},
            FontWeight::{self, *},
            Fonts, LineHeight,
        },
        Catppuccin,
    };

    pub fn fonts() -> Fonts {
        const EMOJI: &str = "/System/Library/Fonts/Apple Color Emoji.ttc";
        const FOLDER: &str = "/Users/romain/Library/Fonts/";
        const FONTS: &[(&str, &[(&str, FontWeight, FontStyle)])] = &[
            (
                "Victor",
                &[
                    // Normal
                    ("VictorMono-Thin.ttf", Thin, Normal),
                    ("VictorMono-ExtraLight.ttf", ExtraLight, Normal),
                    ("VictorMono-Light.ttf", Light, Normal),
                    ("VictorMono-Regular.ttf", Regular, Normal),
                    ("VictorMono-Medium.ttf", Medium, Normal),
                    ("VictorMono-SemiBold.ttf", SemiBold, Normal),
                    ("VictorMono-Bold.ttf", Bold, Normal),
                    // Italic
                    ("VictorMono-ThinItalic.ttf", Thin, Italic),
                    ("VictorMono-ExtraLightItalic.ttf", ExtraLight, Italic),
                    ("VictorMono-LightItalic.ttf", Light, Italic),
                    ("VictorMono-Italic.ttf", Regular, Italic),
                    ("VictorMono-MediumItalic.ttf", Medium, Italic),
                    ("VictorMono-SemiBoldItalic.ttf", SemiBold, Italic),
                    ("VictorMono-BoldItalic.ttf", Bold, Italic),
                    // Oblique
                    ("VictorMono-ThinOblique.ttf", Thin, Oblique),
                    ("VictorMono-ExtraLightOblique.ttf", ExtraLight, Oblique),
                    ("VictorMono-LightOblique.ttf", Light, Oblique),
                    ("VictorMono-Oblique.ttf", Regular, Oblique),
                    ("VictorMono-MediumOblique.ttf", Medium, Oblique),
                    ("VictorMono-SemiBoldOblique.ttf", SemiBold, Oblique),
                    ("VictorMono-BoldOblique.ttf", Bold, Oblique),
                ],
            ),
            (
                "JetBrains",
                &[
                    // Normal
                    ("JetBrainsMonoNerdFont-Thin.ttf", Thin, Normal),
                    ("JetBrainsMonoNerdFont-ExtraLight.ttf", ExtraLight, Normal),
                    ("JetBrainsMonoNerdFont-Light.ttf", Light, Normal),
                    ("JetBrainsMonoNerdFont-Regular.ttf", Regular, Normal),
                    ("JetBrainsMonoNerdFont-Medium.ttf", Medium, Normal),
                    ("JetBrainsMonoNerdFont-SemiBold.ttf", SemiBold, Normal),
                    ("JetBrainsMonoNerdFont-Bold.ttf", Bold, Normal),
                    ("JetBrainsMonoNerdFont-ExtraBold.ttf", ExtraBold, Normal),
                    // Italic
                    ("JetBrainsMonoNerdFont-ThinItalic.ttf", Thin, Italic),
                    (
                        "JetBrainsMonoNerdFont-ExtraLightItalic.ttf",
                        ExtraLight,
                        Italic,
                    ),
                    ("JetBrainsMonoNerdFont-LightItalic.ttf", Light, Italic),
                    ("JetBrainsMonoNerdFont-Italic.ttf", Regular, Italic),
                    ("JetBrainsMonoNerdFont-MediumItalic.ttf", Medium, Italic),
                    ("JetBrainsMonoNerdFont-SemiBoldItalic.ttf", SemiBold, Italic),
                    ("JetBrainsMonoNerdFont-BoldItalic.ttf", Bold, Italic),
                    (
                        "JetBrainsMonoNerdFont-ExtraBoldItalic.ttf",
                        ExtraBold,
                        Italic,
                    ),
                ],
            ),
        ];

        let mut fonts = Fonts::new(Font::from_file(EMOJI).unwrap());

        for (family, faces) in FONTS {
            let family = fonts.set(String::from(*family)).unwrap();

            for (font, font_weight, font_style) in *faces {
                let font = fonts
                    .set(Font::from_file(String::from(FOLDER) + font).unwrap())
                    .unwrap();

                fonts
                    .set((family, *font_weight, *font_style, font))
                    .unwrap();
            }
        }

        fonts
    }

    pub fn ui_theme(context: &Context) -> UiTheme {
        let catppuccin = Catppuccin::default();
        let family = context.fonts().get("Victor").unwrap().key();
        let font_size = 15 as FontSize;
        let line_height = font_size as LineHeight + font_size as LineHeight / 4;

        UiTheme {
            syntax: SyntaxTheme::catppuccin(),
            background_color: catppuccin.base.solid(),
            inactive_foreground_color: catppuccin.mantle.solid().transparent(255 / 3),

            family,
            font_size,
            line_height,
            advance: context
                .fonts()
                .get((family, FontWeight::default(), FontStyle::default()))
                .unwrap()
                .advance_for_size(font_size),

            scroll_duration: Duration::from_millis(500),
            scroll_tween: Tween::ExpoOut,
            scrollbar_color: catppuccin.surface1.solid(),

            normal_mode_color: catppuccin.blue.solid(),
            insert_mode_color: catppuccin.pink.solid(),

            caret_width: 2,
        }
    }
}
