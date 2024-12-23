mod highlighteds;
pub mod panes;
pub mod theme;
pub mod tween;
pub mod ui;
pub mod views {
    mod document;
    mod files;
    mod status;

    pub use document::*;
    pub use files::*;
    pub use status::*;
}

use highlighteds::Highlighted;
use std::collections::HashMap;
use swash::{scale::ScaleContext, shape::ShapeContext};
use theme::UiTheme;
use virus_editor::document::DocumentId;
use virus_graphics::text::Fonts;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Context                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Rendering context.
pub struct Context {
    /// Font cache.
    pub fonts: Fonts,
    /// Shape context.
    pub shape: ShapeContext,
    /// Scale context.
    pub scale: ScaleContext,
    /// UI theme.
    pub theme: UiTheme,
    /// Highlighted documents cache.
    pub highlighteds: HashMap<DocumentId, Highlighted>,
}

impl Context {
    pub fn as_mut(&mut self) -> ContextMut {
        ContextMut {
            fonts: &mut self.fonts,
            shape: &mut self.shape,
            scale: &mut self.scale,
            theme: &mut self.theme,
            highlighteds: &mut self.highlighteds,
        }
    }
}

pub struct ContextMut<'a> {
    pub fonts: &'a Fonts,
    pub shape: &'a mut ShapeContext,
    pub scale: &'a mut ScaleContext,
    pub theme: &'a UiTheme,
    pub highlighteds: &'a mut HashMap<DocumentId, Highlighted>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              TODO                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[allow(unused)]
mod todo {
    use crate::{
        theme::{SyntaxTheme, UiTheme},
        tween::Tween,
    };
    use std::time::Duration;
    use virus_graphics::{
        text::{
            Font, FontSize,
            FontStyle::{self, *},
            FontWeight::{self, *},
            Fonts, Styles,
        },
        types::Rgba,
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

    pub fn ui_theme(fonts: &Fonts) -> UiTheme {
        let catppuccin = Catppuccin::latte();
        let family = fonts.get("Victor").unwrap().key();
        let font_size = 15 as FontSize;
        let line_height = font_size as u32 * 5 / 4;

        UiTheme {
            syntax: catppuccin.syntax_theme(),
            background_color: catppuccin.base.solid(),
            inactive_foreground_color: catppuccin.mantle.solid().transparent(255 / 3),

            family,
            font_size,
            line_height,
            advance: fonts
                .get((family, FontWeight::default(), FontStyle::default()))
                .unwrap()
                .advance_for_size(font_size),

            scroll_duration: Duration::from_millis(500),
            scroll_tween: Tween::ExpoOut,
            scrollbar_color: catppuccin.surface1.solid(),

            normal_mode_color: catppuccin.blue.solid(),
            insert_mode_color: catppuccin.pink.solid(),

            caret_width: 2,

            status_background_color: catppuccin.crust.solid(),
            status_mode_foreground_color: catppuccin.base.solid(),
            status_file_foreground_color: catppuccin.text.solid(),
        }
    }

    #[derive(Copy, Clone, Debug)]
    struct Catppuccin {
        rosewater: Rgba,
        flamingo: Rgba,
        pink: Rgba,
        mauve: Rgba,
        red: Rgba,
        maroon: Rgba,
        peach: Rgba,
        yellow: Rgba,
        green: Rgba,
        teal: Rgba,
        sky: Rgba,
        sapphire: Rgba,
        blue: Rgba,
        lavender: Rgba,
        text: Rgba,
        subtext1: Rgba,
        subtext0: Rgba,
        overlay2: Rgba,
        overlay1: Rgba,
        overlay0: Rgba,
        surface2: Rgba,
        surface1: Rgba,
        surface0: Rgba,
        base: Rgba,
        mantle: Rgba,
        crust: Rgba,
    }

    impl Catppuccin {
        fn syntax_theme(self) -> SyntaxTheme {
            fn style(foreground: Rgba, weight: FontWeight, style: FontStyle) -> Styles {
                Styles {
                    weight,
                    style,
                    foreground,
                    ..Default::default()
                }
            }

            SyntaxTheme {
                default: style(self.text, Black, Normal),
                attribute: style(self.yellow, Black, Normal),
                comment: style(self.overlay2, Black, Italic),
                constant: style(self.peach, Black, Normal),
                constant_builtin_boolean: style(self.pink, Black, Normal),
                constant_character: style(self.teal, Black, Normal),
                constant_character_escape: style(self.pink, Black, Normal),
                constant_numeric_float: style(self.pink, Black, Normal),
                constant_numeric_integer: style(self.pink, Black, Normal),
                constructor: style(self.sapphire, Black, Normal),
                function: style(self.blue, Black, Normal),
                function_macro: style(self.mauve, Black, Normal),
                function_method: style(self.mauve, Black, Normal),
                keyword: style(self.mauve, Black, Normal),
                keyword_control: style(self.mauve, Black, Normal),
                keyword_control_conditional: style(self.mauve, Black, Normal),
                keyword_control_import: style(self.mauve, Black, Normal),
                keyword_control_repeat: style(self.mauve, Black, Normal),
                keyword_control_return: style(self.mauve, Black, Normal),
                keyword_function: style(self.mauve, Black, Normal),
                keyword_operator: style(self.mauve, Black, Normal),
                keyword_special: style(self.mauve, Black, Normal),
                keyword_storage: style(self.mauve, Black, Normal),
                keyword_storage_modifier: style(self.mauve, Black, Normal),
                keyword_storage_modifier_mut: style(self.mauve, Black, Normal),
                keyword_storage_modifier_ref: style(self.mauve, Black, Normal),
                keyword_storage_type: style(self.mauve, Black, Normal),
                label: style(self.sapphire, Black, Normal),
                namespace: style(self.yellow, Black, Normal),
                operator: style(self.sky, Black, Normal),
                punctuation_bracket: style(self.overlay2, Black, Normal),
                punctuation_delimiter: style(self.sky, Black, Normal),
                special: style(self.blue, Black, Normal),
                string: style(self.green, Black, Normal),
                r#type: style(self.yellow, Black, Normal),
                type_builtin: style(self.yellow, Black, Normal),
                type_enum_variant: style(self.teal, Black, Normal),
                type_parameter: style(self.yellow, Black, Normal),
                variable: style(self.text, Black, Normal),
                variable_builtin: style(self.red, Black, Normal),
                variable_other_member: style(self.teal, Black, Normal),
                variable_parameter: style(self.maroon, Black, Normal),
            }
        }

        fn latte() -> Self {
            Self {
                rosewater: Self::hex_to_rgba("#dc8a78"),
                flamingo: Self::hex_to_rgba("#dd7878"),
                pink: Self::hex_to_rgba("#ea76cb"),
                mauve: Self::hex_to_rgba("#8839ef"),
                red: Self::hex_to_rgba("#d20f39"),
                maroon: Self::hex_to_rgba("#e64553"),
                peach: Self::hex_to_rgba("#fe640b"),
                yellow: Self::hex_to_rgba("#df8e1d"),
                green: Self::hex_to_rgba("#40a02b"),
                teal: Self::hex_to_rgba("#179299"),
                sky: Self::hex_to_rgba("#04a5e5"),
                sapphire: Self::hex_to_rgba("#209fb5"),
                blue: Self::hex_to_rgba("#1e66f5"),
                lavender: Self::hex_to_rgba("#7287fd"),
                text: Self::hex_to_rgba("#4c4f69"),
                subtext1: Self::hex_to_rgba("#5c5f77"),
                subtext0: Self::hex_to_rgba("#6c6f85"),
                overlay2: Self::hex_to_rgba("#7c7f93"),
                overlay1: Self::hex_to_rgba("#8c8fa1"),
                overlay0: Self::hex_to_rgba("#9ca0b0"),
                surface2: Self::hex_to_rgba("#acb0be"),
                surface1: Self::hex_to_rgba("#bcc0cc"),
                surface0: Self::hex_to_rgba("#ccd0da"),
                base: Self::hex_to_rgba("#eff1f5"),
                mantle: Self::hex_to_rgba("#e6e9ef"),
                crust: Self::hex_to_rgba("#dce0e8"),
            }
        }

        fn frappe() -> Self {
            Self {
                rosewater: Self::hex_to_rgba("#f2d5cf"),
                flamingo: Self::hex_to_rgba("#eebebe"),
                pink: Self::hex_to_rgba("#f4b8e4"),
                mauve: Self::hex_to_rgba("#ca9ee6"),
                red: Self::hex_to_rgba("#e78284"),
                maroon: Self::hex_to_rgba("#ea999c"),
                peach: Self::hex_to_rgba("#ef9f76"),
                yellow: Self::hex_to_rgba("#e5c890"),
                green: Self::hex_to_rgba("#a6d189"),
                teal: Self::hex_to_rgba("#81c8be"),
                sky: Self::hex_to_rgba("#99d1db"),
                sapphire: Self::hex_to_rgba("#85c1dc"),
                blue: Self::hex_to_rgba("#8caaee"),
                lavender: Self::hex_to_rgba("#babbf1"),
                text: Self::hex_to_rgba("#c6d0f5"),
                subtext1: Self::hex_to_rgba("#b5bfe2"),
                subtext0: Self::hex_to_rgba("#a5adce"),
                overlay2: Self::hex_to_rgba("#949cbb"),
                overlay1: Self::hex_to_rgba("#838ba7"),
                overlay0: Self::hex_to_rgba("#737994"),
                surface2: Self::hex_to_rgba("#626880"),
                surface1: Self::hex_to_rgba("#51576d"),
                surface0: Self::hex_to_rgba("#414559"),
                base: Self::hex_to_rgba("#303446"),
                mantle: Self::hex_to_rgba("#292c3c"),
                crust: Self::hex_to_rgba("#232634"),
            }
        }

        fn macchiato() -> Self {
            Self {
                rosewater: Self::hex_to_rgba("#f4dbd6"),
                flamingo: Self::hex_to_rgba("#f0c6c6"),
                pink: Self::hex_to_rgba("#f5bde6"),
                mauve: Self::hex_to_rgba("#c6a0f6"),
                red: Self::hex_to_rgba("#ed8796"),
                maroon: Self::hex_to_rgba("#ee99a0"),
                peach: Self::hex_to_rgba("#f5a97f"),
                yellow: Self::hex_to_rgba("#eed49f"),
                green: Self::hex_to_rgba("#a6da95"),
                teal: Self::hex_to_rgba("#8bd5ca"),
                sky: Self::hex_to_rgba("#91d7e3"),
                sapphire: Self::hex_to_rgba("#7dc4e4"),
                blue: Self::hex_to_rgba("#8aadf4"),
                lavender: Self::hex_to_rgba("#b7bdf8"),
                text: Self::hex_to_rgba("#cad3f5"),
                subtext1: Self::hex_to_rgba("#b8c0e0"),
                subtext0: Self::hex_to_rgba("#a5adcb"),
                overlay2: Self::hex_to_rgba("#939ab7"),
                overlay1: Self::hex_to_rgba("#8087a2"),
                overlay0: Self::hex_to_rgba("#6e738d"),
                surface2: Self::hex_to_rgba("#5b6078"),
                surface1: Self::hex_to_rgba("#494d64"),
                surface0: Self::hex_to_rgba("#363a4f"),
                base: Self::hex_to_rgba("#24273a"),
                mantle: Self::hex_to_rgba("#1e2030"),
                crust: Self::hex_to_rgba("#181926"),
            }
        }

        fn mocha() -> Self {
            Self {
                rosewater: Self::hex_to_rgba("#f5e0dc"),
                flamingo: Self::hex_to_rgba("#f2cdcd"),
                pink: Self::hex_to_rgba("#f5c2e7"),
                mauve: Self::hex_to_rgba("#cba6f7"),
                red: Self::hex_to_rgba("#f38ba8"),
                maroon: Self::hex_to_rgba("#eba0ac"),
                peach: Self::hex_to_rgba("#fab387"),
                yellow: Self::hex_to_rgba("#f9e2af"),
                green: Self::hex_to_rgba("#a6e3a1"),
                teal: Self::hex_to_rgba("#94e2d5"),
                sky: Self::hex_to_rgba("#89dceb"),
                sapphire: Self::hex_to_rgba("#74c7ec"),
                blue: Self::hex_to_rgba("#89b4fa"),
                lavender: Self::hex_to_rgba("#b4befe"),
                text: Self::hex_to_rgba("#cdd6f4"),
                subtext1: Self::hex_to_rgba("#bac2de"),
                subtext0: Self::hex_to_rgba("#a6adc8"),
                overlay2: Self::hex_to_rgba("#9399b2"),
                overlay1: Self::hex_to_rgba("#7f849c"),
                overlay0: Self::hex_to_rgba("#6c7086"),
                surface2: Self::hex_to_rgba("#585b70"),
                surface1: Self::hex_to_rgba("#45475a"),
                surface0: Self::hex_to_rgba("#313244"),
                base: Self::hex_to_rgba("#1e1e2e"),
                mantle: Self::hex_to_rgba("#181825"),
                crust: Self::hex_to_rgba("#11111b"),
            }
        }

        fn hex_to_rgba(hex: &str) -> Rgba {
            let (r, g, b) = (
                u8::from_str_radix(&hex[1..3], 16).unwrap(),
                u8::from_str_radix(&hex[3..5], 16).unwrap(),
                u8::from_str_radix(&hex[5..7], 16).unwrap(),
            );

            Rgba::new(r, g, b, u8::MAX)
        }
    }
}
