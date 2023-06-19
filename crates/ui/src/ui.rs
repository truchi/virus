use crate::{
    document_view::DocumentView,
    tween::{Tween, Tweened},
};
use std::time::Duration;
use virus_editor::{document::Document, theme::Theme};
use virus_graphics::{
    text::{Context, Font, Fonts},
    wgpu::Graphics,
};
use winit::window::Window;

const HIGHLIGHT_QUERY: &str = include_str!("../../editor/treesitter/rust/highlights.scm");
const SCROLL_DURATION: Duration = Duration::from_millis(500);

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                 Ui                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Ui {
    graphics: Graphics,
    context: Context,
    document_view: DocumentView,
    scroll_top: Tweened<u32>,
    scrollbar_alpha: Tweened<u8>,
    time: Duration,
}

impl Ui {
    pub fn new(window: Window) -> Self {
        let graphics = Graphics::new(window);
        let context = Context::new(fonts());
        let document_view = DocumentView::new(
            HIGHLIGHT_QUERY.into(),
            context.fonts().get("Victor").unwrap().key(),
            Theme::dracula(),
            16,
            20,
        );

        Self {
            graphics,
            context,
            document_view,
            scroll_top: Tweened::new(0),
            scrollbar_alpha: Tweened::new(0),
            time: Duration::ZERO,
        }
    }

    pub fn window(&self) -> &Window {
        self.graphics.window()
    }

    pub fn is_animating(&self) -> bool {
        self.scroll_top.is_animating() || self.scrollbar_alpha.is_animating()
    }

    pub fn scroll_up(&mut self) {
        let scroll = self.screen_height_in_lines() / 2 * self.document_view.line_height();

        self.scroll_top.to(
            self.scroll_top.end().saturating_sub(scroll),
            SCROLL_DURATION,
            Tween::ExpoOut,
        );
        self.scrollbar_alpha = Tweened::with_animation(255, 0, SCROLL_DURATION, Tween::ExpoOut);
    }

    pub fn scroll_down(&mut self) {
        let line_height = self.document_view.line_height();
        let rope_lines = self.document_view.rope().len_lines() as u32 - 1;
        let screen_height_in_lines = self.screen_height_in_lines();

        if rope_lines > screen_height_in_lines {
            let end = self.scroll_top.end() + screen_height_in_lines / 2 * line_height;
            let end = end.min((rope_lines - screen_height_in_lines) * line_height);

            self.scroll_top.to(end, SCROLL_DURATION, Tween::ExpoOut);
            self.scrollbar_alpha = Tweened::with_animation(255, 0, SCROLL_DURATION, Tween::ExpoOut);
        }
    }

    pub fn resize(&mut self) {
        self.graphics.resize();
    }

    pub fn update(&mut self, delta: Duration) {
        self.scroll_top.step(delta);
        self.scrollbar_alpha.step(delta);
        self.time += delta;
    }

    pub fn render(&mut self, document: &Document) {
        let region = self.region();

        self.document_view.render(
            &mut self.context,
            &mut self.graphics.draw(region),
            document,
            self.scroll_top.current(),
            self.scrollbar_alpha.current(),
            self.time,
        );

        self.graphics.render();
    }
}

/// Private.
impl Ui {
    fn screen_height_in_lines(&self) -> u32 {
        self.window().inner_size().height / self.document_view.line_height()
    }

    fn region(&self) -> ([i32; 2], [u32; 2]) {
        let size = self.window().inner_size();
        let height = self.screen_height_in_lines() * self.document_view.line_height();
        let top = (size.height - height) as i32 / 2;

        ([top, 0], [size.width, height])
    }
}

fn fonts() -> Fonts {
    use virus_graphics::text::AnimatedFont;
    use virus_graphics::text::FontStyle::*;
    use virus_graphics::text::FontWeight::*;

    const EMOJI: &str = "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf";

    // const UBUNTU: &str = "/usr/share/fonts/truetype/ubuntu/Ubuntu-B.ttf";

    const FIRA_LIGHT: &str =
        "/home/romain/.local/share/fonts/FiraCode/Fira Code Light Nerd Font Complete.ttf";
    const FIRA_REGULAR: &str =
        "/home/romain/.local/share/fonts/FiraCode/Fira Code Regular Nerd Font Complete.ttf";
    const FIRA_MEDIUM: &str =
        "/home/romain/.local/share/fonts/FiraCode/Fira Code Medium Nerd Font Complete.ttf";
    const FIRA_BOLD: &str =
        "/home/romain/.local/share/fonts/FiraCode/Fira Code Bold Nerd Font Complete.ttf";

    const RECURSIVE_REGULAR: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Regular-1.085.ttf";
    const RECURSIVE_BOLD: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Bold-1.085.ttf";
    const RECURSIVE_ITALIC: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Italic-1.085.ttf";
    const RECURSIVE_BOLD_ITALIC: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-BoldItalic-1.085.ttf";

    const JETBRAINS_THIN: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-Thin.ttf";
    const JETBRAINS_EXTRA_LIGHT: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-ExtraLight.ttf";
    const JETBRAINS_LIGHT: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-Light.ttf";
    const JETBRAINS_REGULAR: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-Regular.ttf";
    const JETBRAINS_MEDIUM: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-Medium.ttf";
    const JETBRAINS_BOLD: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-Bold.ttf";
    const JETBRAINS_SEMI_BOLD: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-SemiBold.ttf";
    const JETBRAINS_EXTRA_BOLD: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-ExtraBold.ttf";
    const JETBRAINS_THIN_ITALIC: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-ThinItalic.ttf";
    const JETBRAINS_EXTRA_LIGHT_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-ExtraLightItalic.ttf";
    const JETBRAINS_LIGHT_ITALIC: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-LightItalic.ttf";
    const JETBRAINS_ITALIC: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-Italic.ttf";
    const JETBRAINS_MEDIUM_ITALIC: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-MediumItalic.ttf";
    const JETBRAINS_BOLD_ITALIC: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-BoldItalic.ttf";
    const JETBRAINS_SEMI_BOLD_ITALIC: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-SemiBoldItalic.ttf";
    const JETBRAINS_EXTRA_BOLD_ITALIC: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono/fonts/ttf/JetBrainsMono-ExtraBoldItalic.ttf";

    const VICTOR_THIN: &str = "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-Thin.ttf";
    const VICTOR_EXTRA_LIGHT: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-ExtraLight.ttf";
    const VICTOR_LIGHT: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-Light.ttf";
    const VICTOR_REGULAR: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-Regular.ttf";
    const VICTOR_MEDIUM: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-Medium.ttf";
    const VICTOR_SEMI_BOLD: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-SemiBold.ttf";
    const VICTOR_BOLD: &str = "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-Bold.ttf";
    const VICTOR_THIN_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-ThinItalic.ttf";
    const VICTOR_EXTRA_LIGHT_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-ExtraLightItalic.ttf";
    const VICTOR_LIGHT_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-LightItalic.ttf";
    const VICTOR_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-Italic.ttf";
    const VICTOR_MEDIUM_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-MediumItalic.ttf";
    const VICTOR_SEMI_BOLD_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-SemiBoldItalic.ttf";
    const VICTOR_BOLD_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-BoldItalic.ttf";
    const VICTOR_THIN_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-ThinOblique.ttf";
    const VICTOR_EXTRA_LIGHT_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-ExtraLightOblique.ttf";
    const VICTOR_LIGHT_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-LightOblique.ttf";
    const VICTOR_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-Oblique.ttf";
    const VICTOR_MEDIUM_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-MediumOblique.ttf";
    const VICTOR_SEMI_BOLD_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-SemiBoldOblique.ttf";
    const VICTOR_BOLD_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMono/TTF/VictorMono-BoldOblique.ttf";

    let mut fonts = Fonts::new(
        Font::from_file(EMOJI).unwrap(),
        AnimatedFont::new("/home/romain/Desktop/noto_animated_emoji".into()).unwrap(),
    );

    // let ubuntu = fonts.set(Font::from_file(UBUNTU).unwrap());

    let fira_light = fonts.set(Font::from_file(FIRA_LIGHT).unwrap()).unwrap();
    let fira_regular = fonts.set(Font::from_file(FIRA_REGULAR).unwrap()).unwrap();
    let fira_medium = fonts.set(Font::from_file(FIRA_MEDIUM).unwrap()).unwrap();
    let fira_bold = fonts.set(Font::from_file(FIRA_BOLD).unwrap()).unwrap();

    let recursive_regular = fonts
        .set(Font::from_file(RECURSIVE_REGULAR).unwrap())
        .unwrap();
    let recursive_bold = fonts.set(Font::from_file(RECURSIVE_BOLD).unwrap()).unwrap();
    let recursive_italic = fonts
        .set(Font::from_file(RECURSIVE_ITALIC).unwrap())
        .unwrap();
    let recursive_bold_italic = fonts
        .set(Font::from_file(RECURSIVE_BOLD_ITALIC).unwrap())
        .unwrap();

    let jetbrains_thin = fonts.set(Font::from_file(JETBRAINS_THIN).unwrap()).unwrap();
    let jetbrains_extra_light = fonts
        .set(Font::from_file(JETBRAINS_EXTRA_LIGHT).unwrap())
        .unwrap();
    let jetbrains_light = fonts
        .set(Font::from_file(JETBRAINS_LIGHT).unwrap())
        .unwrap();
    let jetbrains_regular = fonts
        .set(Font::from_file(JETBRAINS_REGULAR).unwrap())
        .unwrap();
    let jetbrains_medium = fonts
        .set(Font::from_file(JETBRAINS_MEDIUM).unwrap())
        .unwrap();
    let jetbrains_bold = fonts.set(Font::from_file(JETBRAINS_BOLD).unwrap()).unwrap();
    let jetbrains_semi_bold = fonts
        .set(Font::from_file(JETBRAINS_SEMI_BOLD).unwrap())
        .unwrap();
    let jetbrains_extra_bold = fonts
        .set(Font::from_file(JETBRAINS_EXTRA_BOLD).unwrap())
        .unwrap();
    let jetbrains_thin_italic = fonts
        .set(Font::from_file(JETBRAINS_THIN_ITALIC).unwrap())
        .unwrap();
    let jetbrains_extra_light_italic = fonts
        .set(Font::from_file(JETBRAINS_EXTRA_LIGHT_ITALIC).unwrap())
        .unwrap();
    let jetbrains_light_italic = fonts
        .set(Font::from_file(JETBRAINS_LIGHT_ITALIC).unwrap())
        .unwrap();
    let jetbrains_italic = fonts
        .set(Font::from_file(JETBRAINS_ITALIC).unwrap())
        .unwrap();
    let jetbrains_medium_italic = fonts
        .set(Font::from_file(JETBRAINS_MEDIUM_ITALIC).unwrap())
        .unwrap();
    let jetbrains_bold_italic = fonts
        .set(Font::from_file(JETBRAINS_BOLD_ITALIC).unwrap())
        .unwrap();
    let jetbrains_semi_bold_italic = fonts
        .set(Font::from_file(JETBRAINS_SEMI_BOLD_ITALIC).unwrap())
        .unwrap();
    let jetbrains_extra_bold_italic = fonts
        .set(Font::from_file(JETBRAINS_EXTRA_BOLD_ITALIC).unwrap())
        .unwrap();

    let victor_thin = fonts.set(Font::from_file(VICTOR_THIN).unwrap()).unwrap();
    let victor_extra_light = fonts
        .set(Font::from_file(VICTOR_EXTRA_LIGHT).unwrap())
        .unwrap();
    let victor_light = fonts.set(Font::from_file(VICTOR_LIGHT).unwrap()).unwrap();
    let victor_regular = fonts.set(Font::from_file(VICTOR_REGULAR).unwrap()).unwrap();
    let victor_medium = fonts.set(Font::from_file(VICTOR_MEDIUM).unwrap()).unwrap();
    let victor_semi_bold = fonts
        .set(Font::from_file(VICTOR_SEMI_BOLD).unwrap())
        .unwrap();
    let victor_bold = fonts.set(Font::from_file(VICTOR_BOLD).unwrap()).unwrap();
    let victor_thin_italic = fonts
        .set(Font::from_file(VICTOR_THIN_ITALIC).unwrap())
        .unwrap();
    let victor_extra_light_italic = fonts
        .set(Font::from_file(VICTOR_EXTRA_LIGHT_ITALIC).unwrap())
        .unwrap();
    let victor_light_italic = fonts
        .set(Font::from_file(VICTOR_LIGHT_ITALIC).unwrap())
        .unwrap();
    let victor_italic = fonts.set(Font::from_file(VICTOR_ITALIC).unwrap()).unwrap();
    let victor_medium_italic = fonts
        .set(Font::from_file(VICTOR_MEDIUM_ITALIC).unwrap())
        .unwrap();
    let victor_semi_bold_italic = fonts
        .set(Font::from_file(VICTOR_SEMI_BOLD_ITALIC).unwrap())
        .unwrap();
    let victor_bold_italic = fonts
        .set(Font::from_file(VICTOR_BOLD_ITALIC).unwrap())
        .unwrap();
    let victor_thin_oblique = fonts
        .set(Font::from_file(VICTOR_THIN_OBLIQUE).unwrap())
        .unwrap();
    let victor_extra_light_oblique = fonts
        .set(Font::from_file(VICTOR_EXTRA_LIGHT_OBLIQUE).unwrap())
        .unwrap();
    let victor_light_oblique = fonts
        .set(Font::from_file(VICTOR_LIGHT_OBLIQUE).unwrap())
        .unwrap();
    let victor_oblique = fonts.set(Font::from_file(VICTOR_OBLIQUE).unwrap()).unwrap();
    let victor_medium_oblique = fonts
        .set(Font::from_file(VICTOR_MEDIUM_OBLIQUE).unwrap())
        .unwrap();
    let victor_semi_bold_oblique = fonts
        .set(Font::from_file(VICTOR_SEMI_BOLD_OBLIQUE).unwrap())
        .unwrap();
    let victor_bold_oblique = fonts
        .set(Font::from_file(VICTOR_BOLD_OBLIQUE).unwrap())
        .unwrap();

    let fira = fonts.set(String::from("FiraCode")).unwrap();
    fonts.set((fira, Light, Normal, fira_light)).unwrap();
    fonts.set((fira, Regular, Normal, fira_regular)).unwrap();
    fonts.set((fira, Medium, Normal, fira_medium)).unwrap();
    fonts.set((fira, Bold, Normal, fira_bold)).unwrap();

    let recursive = fonts.set(String::from("Recursive")).unwrap();
    fonts
        .set((recursive, Regular, Normal, recursive_regular))
        .unwrap();
    fonts
        .set((recursive, Bold, Normal, recursive_bold))
        .unwrap();
    fonts
        .set((recursive, Regular, Italic, recursive_italic))
        .unwrap();
    fonts
        .set((recursive, Bold, Italic, recursive_bold_italic))
        .unwrap();

    let jetbrains = fonts.set(String::from("JetBrains")).unwrap();
    fonts
        .set((jetbrains, Thin, Normal, jetbrains_thin))
        .unwrap();
    fonts
        .set((jetbrains, ExtraLight, Normal, jetbrains_extra_light))
        .unwrap();
    fonts
        .set((jetbrains, Light, Normal, jetbrains_light))
        .unwrap();
    fonts
        .set((jetbrains, Regular, Normal, jetbrains_regular))
        .unwrap();
    fonts
        .set((jetbrains, Medium, Normal, jetbrains_medium))
        .unwrap();
    fonts
        .set((jetbrains, SemiBold, Normal, jetbrains_semi_bold))
        .unwrap();
    fonts
        .set((jetbrains, Bold, Normal, jetbrains_bold))
        .unwrap();
    fonts
        .set((jetbrains, ExtraBold, Normal, jetbrains_extra_bold))
        .unwrap();
    fonts
        .set((jetbrains, Thin, Italic, jetbrains_thin_italic))
        .unwrap();
    fonts
        .set((jetbrains, ExtraLight, Italic, jetbrains_extra_light_italic))
        .unwrap();
    fonts
        .set((jetbrains, Light, Italic, jetbrains_light_italic))
        .unwrap();
    fonts
        .set((jetbrains, Regular, Italic, jetbrains_italic))
        .unwrap();
    fonts
        .set((jetbrains, Medium, Italic, jetbrains_medium_italic))
        .unwrap();
    fonts
        .set((jetbrains, SemiBold, Italic, jetbrains_semi_bold_italic))
        .unwrap();
    fonts
        .set((jetbrains, Bold, Italic, jetbrains_bold_italic))
        .unwrap();
    fonts
        .set((jetbrains, ExtraBold, Italic, jetbrains_extra_bold_italic))
        .unwrap();

    let victor = fonts.set(String::from("Victor")).unwrap();
    fonts.set((victor, Thin, Normal, victor_thin)).unwrap();
    fonts
        .set((victor, ExtraLight, Normal, victor_extra_light))
        .unwrap();
    fonts.set((victor, Light, Normal, victor_light)).unwrap();
    fonts
        .set((victor, Regular, Normal, victor_regular))
        .unwrap();
    fonts.set((victor, Medium, Normal, victor_medium)).unwrap();
    fonts
        .set((victor, SemiBold, Normal, victor_semi_bold))
        .unwrap();
    fonts.set((victor, Bold, Normal, victor_bold)).unwrap();
    fonts
        .set((victor, Thin, Italic, victor_thin_italic))
        .unwrap();
    fonts
        .set((victor, ExtraLight, Italic, victor_extra_light_italic))
        .unwrap();
    fonts
        .set((victor, Light, Italic, victor_light_italic))
        .unwrap();
    fonts.set((victor, Regular, Italic, victor_italic)).unwrap();
    fonts
        .set((victor, Medium, Italic, victor_medium_italic))
        .unwrap();
    fonts
        .set((victor, SemiBold, Italic, victor_semi_bold_italic))
        .unwrap();
    fonts
        .set((victor, Bold, Italic, victor_bold_italic))
        .unwrap();
    fonts
        .set((victor, Thin, Oblique, victor_thin_oblique))
        .unwrap();
    fonts
        .set((victor, ExtraLight, Oblique, victor_extra_light_oblique))
        .unwrap();
    fonts
        .set((victor, Light, Oblique, victor_light_oblique))
        .unwrap();
    fonts
        .set((victor, Regular, Oblique, victor_oblique))
        .unwrap();
    fonts
        .set((victor, Medium, Oblique, victor_medium_oblique))
        .unwrap();
    fonts
        .set((victor, SemiBold, Oblique, victor_semi_bold_oblique))
        .unwrap();
    fonts
        .set((victor, Bold, Oblique, victor_bold_oblique))
        .unwrap();

    fonts
}
