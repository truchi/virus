const _MULTILINE: &str = "111
222
222
222
222
333";

const _MULTILINE2: &str = "111
222
333";

const _MULTILINE3: &str = "111
222
333";

mod fps;

use std::borrow::Cow;

use pixels::{Error, Pixels, SurfaceTexture};
use virus_common::{Rgb, Rgba, Style};
use virus_editor::{Document, Highlight, Highlights, Theme};
use virus_graphics::{
    pixels_mut::{PixelsMut, Surface},
    reexports::swash::CacheKey,
    text::{Context, Font, FontSize, Fonts, Line},
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
};

const RECURSIVE_VF: &str =
    "/home/romain/.local/share/fonts/Recursive/Recursive_Desktop/Recursive_VF_1.084.ttf";
const RECURSIVE: &str =
    "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Regular-1.084.ttf";
const UBUNTU: &str = "/usr/share/fonts/truetype/ubuntu/Ubuntu-B.ttf";
const FIRA: &str =
    "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Regular Nerd Font Complete Mono.ttf";
const EMOJI: &str = "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf";

fn main() -> Result<(), Error> {
    let fira = Font::from_file(FIRA).unwrap();
    let ubuntu = Font::from_file(UBUNTU).unwrap();
    let recursive = Font::from_file(RECURSIVE).unwrap();
    let emoji = Font::from_file(EMOJI).unwrap();

    let font = recursive;
    let key = font.key();

    let mut context = Context::new(Fonts::new([font], emoji));

    let mut document = Document::open(std::env::args().nth(1).unwrap()).unwrap();
    document.parse();
    let document_view = DocumentView {
        document,
        font_size: 40,
        line_height_factor: 1.25,
    };

    let event_loop = EventLoop::new();
    let window = {
        let window = WindowBuilder::new()
            .with_title("virus")
            .with_inner_size(PhysicalSize::new(1, 1))
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .build(&event_loop)
            .unwrap();
        // window.set_cursor_visible(false);
        window
    };

    let mut pixels = {
        let PhysicalSize { width, height } = window.inner_size();
        Pixels::new(width, height, SurfaceTexture::new(width, height, &window)).unwrap()
    };

    let mut fps_counter = fps::FpsCounter::new();

    let mut scroll_time = std::time::Instant::now();

    event_loop.run(move |event, _, _control_flow| {
        if let Event::WindowEvent {
            event: WindowEvent::Resized(PhysicalSize { width, height }),
            ..
        } = event
        {
            pixels.resize_surface(width, height).unwrap();
            pixels.resize_buffer(width, height).unwrap();
        }

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let mut pixels_mut = {
                let PhysicalSize { width, height } = window.inner_size();
                PixelsMut::new(width, height, pixels.get_frame_mut())
            };

            for (i, u) in pixels_mut.pixels_mut().iter_mut().enumerate() {
                // *u = match i % 4 {
                //     0 => 200,
                //     1 => 200,
                //     2 => 200,
                //     _ => 255,
                // };
                *u = 100;
            }

            let width = pixels_mut.width();
            let height = pixels_mut.height();

            let mut surface = pixels_mut.surface(0, 0, width, height);
            document_view.render(
                &mut surface,
                &mut context,
                key,
                0,
                // 1500 + { scroll_time.elapsed().as_millis() / 10 } as u32,
                0,
            );

            pixels.render().unwrap();
        }

        // Update internal state and request a redraw
        window.request_redraw();
        fps_counter.tick();
    });
}

struct DocumentView {
    document: Document,
    font_size: FontSize,
    line_height_factor: f32,
}

impl DocumentView {
    fn line_height(&self) -> f32 {
        self.line_height_factor * self.font_size as f32
    }

    fn render(
        &self,
        surface: &mut Surface,
        context: &mut Context,
        font: CacheKey,
        scroll_top: u32,
        scroll_left: u32,
    ) {
        const HIGHLIGHT_QUERY: &str = include_str!("../../editor/treesitter/rust/highlights.scm");

        let line_height_f32 = self.line_height();
        let line_height_i32 = line_height_f32.round() as i32;
        let start_line = (scroll_top as f32 / line_height_f32).floor() as usize;
        let end_line = 1 + start_line + (surface.height() as f32 / line_height_f32).ceil() as usize;

        let rope = self.document.rope();
        let highlights = Highlights::new(
            rope,
            self.document.tree().unwrap().root_node(),
            start_line..end_line,
            self.document.query(HIGHLIGHT_QUERY).unwrap(),
            dracula(),
        );

        let mut prev_line = None;
        let mut shaper = Line::shaper(context, self.font_size);

        for Highlight { start, end, style } in highlights.iter() {
            let line = start.line;
            debug_assert!(start.line == end.line);

            if prev_line != Some(line) {
                if let Some(line) = prev_line {
                    let shaped = shaper.line();
                    surface.draw_line(
                        context,
                        line as i32 * line_height_i32 - scroll_top as i32,
                        0,
                        &shaped,
                        self.font_size,
                    );
                }

                prev_line = Some(line);
                shaper = Line::shaper(context, self.font_size);
            }

            shaper.push(
                // We cow to make sure ligatures are not split between rope chunks
                &Cow::from(rope.get_byte_slice(start.index..end.index).unwrap()),
                font,
                // TODO Theme.default
                style.unwrap_or_default(),
            );
        }

        if let Some(line) = prev_line {
            let shaped = shaper.line();
            surface.draw_line(
                context,
                line as i32 * line_height_i32 - scroll_top as i32,
                0,
                &shaped,
                self.font_size,
            );
        }
    }
}

fn dracula() -> Theme {
    fn style(r: u8, g: u8, b: u8) -> Style {
        Style {
            foreground: Rgba {
                r,
                g,
                b,
                a: u8::MAX,
            },
            ..Default::default()
        }
    }

    let background = style(40, 42, 54);
    let current = style(68, 71, 90);
    let foreground = style(248, 248, 242);
    let comment = style(98, 114, 164);
    let cyan = style(139, 233, 253);
    let green = style(80, 250, 123);
    let orange = style(255, 184, 108);
    let pink = style(255, 121, 198);
    let purple = style(189, 147, 249);
    let red = style(255, 85, 85);
    let yellow = style(241, 250, 140);

    Theme {
        attribute: green,
        comment: current,
        constant: green,
        constant_builtin_boolean: purple,
        constant_character: purple,
        constant_character_escape: purple,
        constant_numeric_float: purple,
        constant_numeric_integer: purple,
        constructor: foreground,
        function: pink,
        function_macro: pink,
        function_method: pink,
        keyword: red,
        keyword_control: red,
        keyword_control_conditional: red,
        keyword_control_import: red,
        keyword_control_repeat: red,
        keyword_control_return: red,
        keyword_function: red,
        keyword_operator: red,
        keyword_special: red,
        keyword_storage: red,
        keyword_storage_modifier: red,
        keyword_storage_modifier_mut: red,
        keyword_storage_modifier_ref: red,
        keyword_storage_type: red,
        label: foreground,
        namespace: foreground,
        operator: foreground,
        punctuation_bracket: yellow,
        punctuation_delimiter: yellow,
        special: yellow,
        string: cyan,
        r#type: cyan,
        type_builtin: cyan,
        type_enum_variant: cyan,
        variable: orange,
        variable_builtin: orange,
        variable_other_member: orange,
        variable_parameter: orange,
    }
}
