#![allow(unused)]

mod buffer;
mod line;

use buffer::*;
use line::*;

use softbuffer::GraphicsContext;
use std::time::{Duration, Instant};
use swash::scale::image::{Content, Image};
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::shape::cluster::Glyph;
use swash::shape::ShapeContext;
use swash::zeno::{Format, Placement};
use swash::{CacheKey, FontRef, GlyphId, Weight};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Window, WindowBuilder};

// const RECURSIVE: &str =
//     "/home/romain/.local/share/fonts/Recursive/Recursive_Desktop/Recursive_VF_1.084.ttf";
const RECURSIVE: &str =
    "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Regular-1.084.ttf";
const FIRA: &str =
    "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Regular Nerd Font Complete Mono.ttf";
const EMOJI: &str = "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf";
const FONT: &str = RECURSIVE;

#[derive(Copy, Clone, Default, Debug)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

// 🔥 🦀
impl Rgb {
    pub const RED: Self = Self::new(255, 0, 0);
    pub const GREEN: Self = Self::new(0, 255, 0);
    pub const BLUE: Self = Self::new(0, 0, 255);
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn grey(grey: u8) -> Self {
        Self {
            r: grey,
            g: grey,
            b: grey,
        }
    }

    pub fn mul(&mut self, by: u8) {
        self.r = (self.r as f32 * (by as f32 / 255.)) as u8;
        self.g = (self.g as f32 * (by as f32 / 255.)) as u8;
        self.b = (self.b as f32 * (by as f32 / 255.)) as u8;
    }
}

impl From<Rgb> for u32 {
    fn from(Rgb { r, g, b }: Rgb) -> Self {
        b as u32 | ((g as u32) << 8) | ((r as u32) << 16)
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(&event_loop)
        .unwrap();

    let mut graphics_context = unsafe { GraphicsContext::new(window) }.unwrap();
    let size = graphics_context.window().inner_size();
    let (width, height) = (size.width as usize, size.height as usize);
    let mut buffer = Buffer::new(width, height, Rgb::default());

    let recursive = Font::from_file(RECURSIVE).unwrap();
    let fira = Font::from_file(FIRA).unwrap();
    let emoji = Font::from_file(EMOJI).unwrap();

    let recursive_key = recursive.key;
    let fira_key = fira.key;
    let emoji_key = emoji.key;

    let mut fonts = Fonts::new(emoji);
    fonts.insert(recursive);
    fonts.insert(fira);

    let mut context = Context::new(fonts);

    const SIZE: FontSize = 60;

    let file = include_str!("./main.rs");
    // let file = "./main.rs";
    let lines = file
        .lines()
        .enumerate()
        .map(|(i, line)| {
            Line::from_iter(
                &mut context,
                [(if i % 2 == 0 { Rgb::RED } else { Rgb::GREEN }, line)],
                // recursive_key,
                fira_key,
                SIZE,
            )
        })
        .collect::<Vec<_>>();

    let now = Instant::now();
    let mut last = now;
    let mut fps = Vec::with_capacity(100);
    let mut fpsi = 0;

    let mut dy = 0;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(window_id) if window_id == graphics_context.window().id() => {
                if now.elapsed() < Duration::from_millis(500) {
                    return;
                }

                let line_height = SIZE as f32 * 1.5;

                if fps.len() == 100 {
                    let sum: f32 = fps.iter().sum();
                    fps.clear();
                    println!("{}", sum / 100.);
                }

                let dur = Instant::now() - last;
                fps.push(1000. / dur.as_millis() as f32);
                last = Instant::now();

                let size = graphics_context.window().inner_size();
                let (width, height) = (size.width as usize, size.height as usize);
                buffer.resize(width, height, Rgb::default());

                dy += 10;
                dy = dy % 10_000;
                for (i, line) in lines.iter().enumerate() {
                    let mut advance = 0;
                    let mut descent = (i + 1) * (line_height) as usize;

                    context.scale(&line, |glyph, image| {
                        if let Some(image) = image {
                            buffer.draw_image_mask(
                                (advance) as _,
                                (descent as i32 - dy as i32) as _,
                                image,
                                glyph.color,
                            );
                        }

                        advance += glyph.advance as usize;

                        true
                    });
                }

                buffer.render(&mut graphics_context);
            }
            Event::MainEventsCleared => {
                graphics_context.window().request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == graphics_context.window().id() => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
