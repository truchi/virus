use crate::muck;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Rgb                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An `RGB` color.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const RED: Self = Self::new(255, 0, 0);
    pub const GREEN: Self = Self::new(0, 255, 0);
    pub const BLUE: Self = Self::new(0, 0, 255);
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const GREY: Self = Self::grey(127);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn grey(grey: u8) -> Self {
        Self::new(grey, grey, grey)
    }

    pub const fn transparent(&self, a: u8) -> Rgba {
        Rgba::new(self.r, self.g, self.b, a)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Rgba                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

muck!(unsafe Rgba => Uint8x4);

/// An `RGBA` color.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const RED: Self = Rgb::RED.transparent(255);
    pub const GREEN: Self = Rgb::GREEN.transparent(255);
    pub const BLUE: Self = Rgb::BLUE.transparent(255);
    pub const BLACK: Self = Rgb::BLACK.transparent(255);
    pub const WHITE: Self = Rgb::WHITE.transparent(255);
    pub const GREY: Self = Rgb::GREY.transparent(255);
    pub const TRANSPARENT: Self = Rgb::BLACK.transparent(0);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn grey(grey: u8, a: u8) -> Self {
        Self::new(grey, grey, grey, a)
    }

    pub const fn solid(&self) -> Rgb {
        Rgb::new(self.r, self.g, self.b)
    }

    pub fn is_visible(&self) -> bool {
        self.a != 0
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Catppuccin                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// For convenience.
pub struct Catppuccin {
    pub rosewater: Rgba,
    pub flamingo: Rgba,
    pub pink: Rgba,
    pub mauve: Rgba,
    pub red: Rgba,
    pub maroon: Rgba,
    pub peach: Rgba,
    pub yellow: Rgba,
    pub green: Rgba,
    pub teal: Rgba,
    pub sky: Rgba,
    pub sapphire: Rgba,
    pub blue: Rgba,
    pub lavender: Rgba,
    pub text: Rgba,
    pub subtext1: Rgba,
    pub subtext0: Rgba,
    pub overlay2: Rgba,
    pub overlay1: Rgba,
    pub overlay0: Rgba,
    pub surface2: Rgba,
    pub surface1: Rgba,
    pub surface0: Rgba,
    pub base: Rgba,
    pub mantle: Rgba,
    pub crust: Rgba,
}

impl Default for Catppuccin {
    fn default() -> Self {
        Self::latte()
    }
}

#[allow(unused)]
impl Catppuccin {
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
