use crate::Rgb;
use softbuffer::GraphicsContext;
use swash::scale::image::{Content, Image};
use winit::window::Window;

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

    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Rgb) {
        let x1 = self.clamp_x(x) as usize;
        let y1 = self.clamp_y(y) as usize;
        let x2 = self.clamp_x(x + w as i32) as usize;
        let y2 = self.clamp_y(y + h as i32) as usize;
        // let x2 = (x + w as i32).min(self.width as i32) as usize;
        // let y2 = (y + h as i32).min(self.height as i32) as usize;

        for y in y1..y2 {
            self.pixels
                .get_mut(x1 + y * self.width..x2 + y * self.width)
                .unwrap()
                .fill(color.into());
        }
    }

    // pub fn draw_image(&mut self, x: i32, y: i32, image: Image) {
    //     match image.content {
    //         Content::SubpixelMask => unreachable!(),
    //         Content::Mask => self.draw_image_mask(x, y, image),
    //         Content::Color => self.draw_image_color(x, y, image),
    //     }
    // }

    pub fn draw_image_mask(&mut self, x: i32, y: i32, image: &Image, color: Rgb) {
        debug_assert!(image.content == Content::Mask);
        let top = image.placement.top;
        let left = image.placement.left;
        let width = image.placement.width as i32;
        let height = image.placement.height as i32;

        // self.draw_rect(x, y, width as _, height as _, Rgb::GREEN);

        let x = x + left;
        let y = y - top;

        // self.draw_rect(x, y, width as _, height as _, Rgb::BLUE);

        let x1 = self.clamp_x(x);
        let y1 = self.clamp_y(y);
        let x2 = self.clamp_x(x + width);
        let y2 = self.clamp_y(y + height);

        for j in y1..y2 {
            for i in x1..x2 {
                let buffer_pixel = self
                    .pixels
                    .get_mut(i as usize + j as usize * self.width)
                    .unwrap();

                let i = i - x;
                let j = j - y;
                let image_pixel = image
                    .data
                    .get(i as usize + j as usize * width as usize)
                    .unwrap();

                if *image_pixel == 0 {
                    continue;
                }

                let mut color = color;
                color.mul(*image_pixel);

                *buffer_pixel = color.into();
            }
        }
    }

    fn draw_image_color(&mut self, x: i32, y: i32, image: Image) {
        debug_assert!(image.content == Content::Color);
    }

    fn clamp_x(&self, x: i32) -> i32 {
        x.max(0).min(self.width as i32)
    }

    fn clamp_y(&self, y: i32) -> i32 {
        y.max(0).min(self.height as i32)
    }
}

impl AsRef<[u32]> for Buffer {
    fn as_ref(&self) -> &[u32] {
        &self.pixels
    }
}
