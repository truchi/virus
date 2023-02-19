use pixels::{Error, Pixels, PixelsBuilder, SurfaceTexture};
use virus_editor::Document;
use virus_graphics::pixels_mut::PixelsMut;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Fullscreen, Window, WindowBuilder};

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let window = {
        let window = WindowBuilder::new()
            .with_title("virus")
            .with_inner_size(PhysicalSize::new(1, 1))
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .build(&event_loop)
            .unwrap();
        window.set_cursor_visible(false);
        window
    };

    let mut pixels = {
        let PhysicalSize { width, height } = window.inner_size();
        Pixels::new(width, height, SurfaceTexture::new(width, height, &window)).unwrap()
    };

    let document =
        Document::open("/home/romain/perso/virus/crates/virus/src/main.rs".into()).unwrap();

    let mut fps_counter = fps::FpsCounter::new();

    event_loop.run(move |event, _, control_flow| {
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
            let pixels_mut = {
                let PhysicalSize { width, height } = window.inner_size();
                PixelsMut::new(width, height, pixels.get_frame_mut())
            };

            for (i, u) in pixels.get_frame_mut().iter_mut().enumerate() {
                *u = match i % 4 {
                    0 => 255,
                    1 => 0,
                    2 => 0,
                    _ => 255,
                };
            }

            pixels.render().unwrap();
        }

        // Update internal state and request a redraw
        window.request_redraw();
        fps_counter.tick();
    });
}

mod fps {
    use std::time::Instant;

    pub struct FpsCounter {
        array: [u128; Self::CAP],
        len: usize,
        now: Instant,
    }

    impl FpsCounter {
        const CAP: usize = 120;

        pub fn new() -> Self {
            Self {
                array: [0; Self::CAP],
                len: 0,
                now: Instant::now(),
            }
        }

        pub fn tick(&mut self) {
            if self.len == Self::CAP {
                let mpf = self.array.iter().sum::<u128>() as f32 / Self::CAP as f32;
                let fps = (1_000_000. / mpf).round();

                println!("{fps}fps");
                self.len = 0;
            }

            self.array[self.len] = self.now.elapsed().as_micros();
            self.len += 1;
            self.now = Instant::now();
        }
    }
}
