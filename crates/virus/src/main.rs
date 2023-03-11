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

use pixels::{Error, Pixels, SurfaceTexture};
use virus_common::{Rgb, Rgba};
use virus_editor::Document;
use virus_graphics::{
    pixels_mut::PixelsMut,
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

const FONT_SIZE: u8 = 60;

fn main() -> Result<(), Error> {
    let fira = Font::from_file(FIRA).unwrap();
    let emoji = Font::from_file(EMOJI).unwrap();
    let ubuntu = Font::from_file(UBUNTU).unwrap();
    let recursive = Font::from_file(RECURSIVE).unwrap();

    let font = recursive;
    let key = font.key();

    let mut context = Context::new(Fonts::new([font], emoji));
    let mut shaper = Line::shaper(&mut context, FONT_SIZE);
    shaper.push("VA Hello world e é c ç", key, Rgb::new(0, 0, 0));
    let line = shaper.line();
    let mut scaler = line.scaler(&mut context);
    let scaled = {
        let mut scaled = vec![];

        while let Some((advance, glyph, image)) = scaler.next() {
            scaled.push((advance, glyph, image.cloned()));
        }

        scaled
    };

    let document = Document::open(std::env::args().nth(1).unwrap()).unwrap();

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
                *u = match i % 4 {
                    0 => 255,
                    1 => 0,
                    2 => 0,
                    _ => 255,
                };
            }

            let width = pixels_mut.width();
            let height = pixels_mut.height();

            if height > 1000 {
                let line_1 = pixels_mut
                    .pixels_mut()
                    .get_mut(4 * 100 * width as usize..4 * 101 * width as usize)
                    .unwrap();
                for p in line_1 {
                    *p = 0;
                }
                let line_2 = pixels_mut
                    .pixels_mut()
                    .get_mut(
                        4 * ((100 + FONT_SIZE as usize) * width as usize)
                            ..4 * ((101 + FONT_SIZE as usize) * width as usize),
                    )
                    .unwrap();
                for p in line_2 {
                    *p = 0;
                }

                for i in 0..FONT_SIZE as usize {
                    *pixels_mut
                        .pixels_mut()
                        .get_mut(4 * (100 + (100 + i) * width as usize))
                        .unwrap() = 0;
                }
            }

            let mut surface = pixels_mut.surface(100, 100, 10_000, 10_000);

            for (advance, glyph, image) in &scaled {
                let image = if let Some(image) = image {
                    image
                } else {
                    continue;
                };
                // We have to render at the baseline!
                surface.draw_image(
                    FONT_SIZE as i32,
                    *advance as i32,
                    image,
                    glyph.color.with_alpha(255),
                );
            }

            pixels.render().unwrap();
        }

        // Update internal state and request a redraw
        window.request_redraw();
        fps_counter.tick();
    });
}

struct DocumentView {
    document: Document,
    scroll_top: u32,
    scroll_left: u32,
    font_size: FontSize,
}

impl DocumentView {
    fn render(&self) {}
}
