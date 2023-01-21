#![deny(clippy::all)]
#![forbid(unsafe_code)]

use pixels::{Error, Pixels, SurfaceTexture};
use std::time::Instant;
use tiny_skia::Pixmap;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod shape;

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello tiny-skia")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);

        let start = Instant::now();
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
        println!("Took {:?} to init wgpu :'(", start.elapsed());

        pixels
    };

    let mut drawing = Pixmap::new(WIDTH, HEIGHT).unwrap();
    let now = Instant::now();

    let mut fps_counter = fps::FpsCounter::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            pixels.get_frame_mut().copy_from_slice(drawing.data());
            if let Err(err) = pixels.render() {
                eprintln!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    eprintln!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            shape::draw(&mut drawing, now.elapsed().as_secs_f32());
            window.request_redraw();
        }

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
