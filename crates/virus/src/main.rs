#![allow(unused)]

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
use virus_common::{Rgba, Style};
use virus_editor::{Document, Highlight, Highlights, Theme};
use virus_graphics::{
    pixels_mut::{PixelsMut, Surface},
    reexports::swash::CacheKey,
    text::{Context, Font, FontSize, Fonts, Line},
};
use virus_ui::document_view::DocumentView;
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

const SCALE: u32 = 1;

const HIGHLIGHT_QUERY: &str = include_str!("../../editor/treesitter/rust/highlights.scm");

// 🚀
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
    let mut document_view = DocumentView::new(HIGHLIGHT_QUERY.into(), dracula(), key, 40, 50);

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

    let scroll_time = std::time::Instant::now();

    event_loop.run(move |event, _, _control_flow| {
        if let Event::WindowEvent {
            event: WindowEvent::Resized(PhysicalSize { width, height }),
            ..
        } = event
        {
            if width != 1 {
                pixels.resize_surface(width, height).unwrap();
                pixels.resize_buffer(width / SCALE, height / SCALE).unwrap();
            }
        }

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let mut pixels_mut = {
                let PhysicalSize { width, height } = window.inner_size();
                PixelsMut::new(width / SCALE, height / SCALE, pixels.get_frame_mut())
            };

            for (i, u) in pixels_mut.pixels_mut().iter_mut().enumerate() {
                *u = match i % 4 {
                    0 => 0,
                    1 => 0,
                    2 => 0,
                    _ => 255,
                };
            }

            let width = pixels_mut.width();
            let height = pixels_mut.height();

            if pixels_mut.pixels().len() == 4 {
                return;
            }

            let scroll_top = 100 + { scroll_time.elapsed().as_millis() / 10 } as u32;
            document_view.render(
                &mut pixels_mut.surface(0, 0, width, height),
                &mut context,
                &document,
                scroll_top,
            );

            pixels.render().unwrap();
        }

        // Update internal state and request a redraw
        window.request_redraw();
        fps_counter.tick();
    });
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

fn uni(r: u8, g: u8, b: u8, a: u8) -> Theme {
    let current = Style {
        foreground: Rgba { r, g, b, a },
        background: Rgba {
            r: u8::MAX - r,
            g: u8::MAX - g,
            b: u8::MAX - b,
            a,
        },
        ..Default::default()
    };

    Theme {
        default: current,
        attribute: current,
        comment: current,
        constant: current,
        constant_builtin_boolean: current,
        constant_character: current,
        constant_character_escape: current,
        constant_numeric_float: current,
        constant_numeric_integer: current,
        constructor: current,
        function: current,
        function_macro: current,
        function_method: current,
        keyword: current,
        keyword_control: current,
        keyword_control_conditional: current,
        keyword_control_import: current,
        keyword_control_repeat: current,
        keyword_control_return: current,
        keyword_function: current,
        keyword_operator: current,
        keyword_special: current,
        keyword_storage: current,
        keyword_storage_modifier: current,
        keyword_storage_modifier_mut: current,
        keyword_storage_modifier_ref: current,
        keyword_storage_type: current,
        label: current,
        namespace: current,
        operator: current,
        punctuation_bracket: current,
        punctuation_delimiter: current,
        special: current,
        string: current,
        r#type: current,
        type_builtin: current,
        type_enum_variant: current,
        variable: current,
        variable_builtin: current,
        variable_other_member: current,
        variable_parameter: current,
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
            // background: Rgba {
            //     r: u8::MAX - r,
            //     g: u8::MAX - g,
            //     b: u8::MAX - b,
            //     a: u8::MAX,
            // },
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
        default: style(255, 255, 255),
        attribute: green,
        comment,
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
