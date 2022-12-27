#![allow(unused)]

mod line;

use line::*;

use softbuffer::GraphicsContext;
use std::time::{Duration, Instant};
use swash::scale::image::Image;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::shape::cluster::Glyph;
use swash::shape::ShapeContext;
use swash::zeno::Format;
use swash::{CacheKey, FontRef, GlyphId, Weight};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Window, WindowBuilder};

const RECURSIVE: &str =
    "/home/romain/.local/share/fonts/Recursive/Recursive_Desktop/Recursive_VF_1.084.ttf";
const FIRA: &str =
    "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Regular Nerd Font Complete Mono.ttf";
const EMOJI: &str = "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf";
const FONT: &str = RECURSIVE;

fn default<T: Default>() -> T {
    T::default()
}

#[derive(Copy, Clone, Default)]
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
}

impl From<Rgb> for u32 {
    fn from(Rgb { r, g, b }: Rgb) -> Self {
        b as u32 | ((g as u32) << 8) | ((r as u32) << 16)
    }
}

pub struct Buffer {
    pixels: Vec<u32>,
    width: usize,
    height: usize,
}

impl Buffer {
    pub fn new(width: usize, height: usize, color: Rgb) -> Self {
        let mut pixels = Vec::with_capacity(width * height);
        pixels.resize(width * height, color.into());

        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize, color: Rgb) {
        self.width = width;
        self.height = height;
        self.pixels.fill(color.into());
        self.pixels.resize(width * height, color.into());
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut u32 {
        &mut self.pixels[y * self.width + x]
    }

    pub fn render(&self, context: &mut GraphicsContext<Window>) {
        context.set_buffer(&self.pixels, self.width as u16, self.height as u16);
    }
}

impl AsRef<[u32]> for Buffer {
    fn as_ref(&self) -> &[u32] {
        &self.pixels
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

    let context = Context::new(fonts);

    struct FontStuff {
        shape_context: ShapeContext,
        scale_context: ScaleContext,
    }

    impl FontStuff {
        fn do_stuff(
            &mut self,
            font: FontRef,
            size: f32,
            str: &str,
            mono: f32,
            wght: f32,
            slnt: f32,
            casl: f32,
            crsv: bool,
        ) -> Vec<(Glyph, Image)> {
            let mut shaper = self
                .shape_context
                .builder(font)
                .size(size)
                .features([("dlig", 1), ("calt", 1)])
                .variations([
                    ("MONO", mono),
                    ("wght", wght),
                    ("slnt", slnt),
                    ("CASL", casl),
                    ("CRSV", if crsv { 1. } else { 0. }),
                ])
                .build();
            let mut scaler = self
                .scale_context
                .builder(font)
                .hint(false)
                .size(size)
                .variations([
                    ("MONO", mono),
                    ("wght", wght),
                    ("slnt", slnt),
                    ("CASL", casl),
                    ("CRSV", if crsv { 1. } else { 0. }),
                ])
                .build();
            let render = Render::new(&[
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Outline,
                Source::Bitmap(StrikeWith::BestFit),
            ]);

            dbg!(scaler.has_bitmaps());
            dbg!(scaler.has_outlines());
            dbg!(scaler.has_color_bitmaps());
            dbg!(scaler.has_color_outlines());

            shaper.add_str(str);

            let mut scaleds: Vec<(Glyph, Image)> = vec![];

            shaper.shape_with(|cluster| {
                for glyph in cluster.glyphs {
                    scaleds.push((*glyph, render.render(&mut scaler, glyph.id).unwrap()));
                }
            });

            scaleds
        }
    }

    let mut font_stuff = FontStuff {
        shape_context: ShapeContext::new(),
        scale_context: ScaleContext::new(),
    };

    let now = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(window_id) if window_id == graphics_context.window().id() => {
                if now.elapsed() < Duration::from_secs(1) {
                    return;
                }

                let size = graphics_context.window().inner_size();
                let (width, height) = (size.width as usize, size.height as usize);

                buffer.resize(width, height, Rgb::default());

                let color = |grey| u32::from(Rgb::grey(grey));

                const X: usize = 100;
                const Y: usize = 100;

                let mono = (now.elapsed().as_millis() % 1_000) as f32 / 1_000.;
                let mono = 1.;
                let weight = (now.elapsed().as_millis() % 1_000) as f32;
                // let weight = 300.;
                let slant = ((now.elapsed().as_millis() % 1_000) as f32 / 1_000.) * 15. - 15.;
                // let slant = 0.;
                let casual = (now.elapsed().as_millis() % 1_000) as f32 / 1_000.;
                // let casual = 0.;
                let cursive = (now.elapsed().as_millis() % 1_000) > 500;
                // let cursive = false;

                let d = format!(
                    "-> mono: {mono}, slant: {:02}, weight: {weight:04}, casual: {casual:05}, cursive: {cursive}",
                    (-slant as isize) as usize,
                );

                let str = include_str!("./main.rs");
                let lines = d.lines().chain(str.lines().skip(10).take(20));
                // let lines = "🦀".lines();

                for (line_offset, str) in lines.enumerate() {
                    let line = Line::from_iter(&mut self, context, iter, size);

                    // let line_offset = line_offset * (60. * 1.5) as usize;
                    // let scaleds = font_stuff.do_stuff(
                    //     font.as_ref(),
                    //     60.,
                    //     &str,
                    //     mono,
                    //     weight,
                    //     slant,
                    //     casual,
                    //     cursive,
                    // );

                    // let mut advance = 0;
                    // for (glyph, image) in &scaleds {
                    //     let gw = image.placement.width as usize;
                    //     let gh = image.placement.height as usize;
                    //     let gt = image.placement.top as isize;
                    //     let gl = image.placement.left as isize;

                    //     for y in 0..gh {
                    //         for x in 0..gw {
                    //             *buffer.get_mut(
                    //                 ((X + advance + x) as isize + gl) as usize,
                    //                 ((Y + line_offset + y) as isize - gt) as usize,
                    //             ) = color(image.data[y * gw + x]);
                    //         }
                    //     }

                    //     advance += glyph.advance as usize;
                    // }
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
