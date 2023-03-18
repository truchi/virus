use crate::text::{Context, Line};
use swash::scale::image::{Content, Image};
use virus_common::{Rgb, Rgba};

#[derive(Debug)]
pub struct PixelsMut<'pixels> {
    width: u32,
    height: u32,
    pixels: &'pixels mut [u8],
}

impl<'pixels> PixelsMut<'pixels> {
    pub fn new(width: u32, height: u32, pixels: &'pixels mut [u8]) -> Self {
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn clamp_top(&self, top: i32) -> i32 {
        top.max(0).min(self.height as i32)
    }

    pub fn clamp_left(&self, left: i32) -> i32 {
        left.max(0).min(self.width as i32)
    }

    pub fn clamp(&self, top: i32, left: i32) -> (i32, i32) {
        (self.clamp_top(top), self.clamp_left(left))
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn surface(&'pixels mut self, top: i32, left: i32, width: u32, height: u32) -> Surface {
        Surface {
            pixels: self,
            top,
            left,
            width,
            height,
        }
    }

    pub fn pixel_mut(&mut self, top: u32, left: u32) -> Option<&mut [u8]> {
        let top = top as usize;
        let left = left as usize;
        let width = self.width as usize;
        let index = left + top * width;

        self.pixels.get_mut(4 * index..4 * (index + 1))
    }
}

pub struct Surface<'pixels> {
    pixels: &'pixels mut PixelsMut<'pixels>,
    top: i32,
    left: i32,
    width: u32,
    height: u32,
}

impl<'pixels> Surface<'pixels> {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn clamp_rel_top(&self, top: i32) -> i32 {
        top.max(0).min(self.height as i32)
    }

    pub fn clamp_rel_left(&self, left: i32) -> i32 {
        left.max(0).min(self.width as i32)
    }

    pub fn clamp_rel(&self, top: i32, left: i32) -> (i32, i32) {
        (self.clamp_rel_top(top), self.clamp_rel_left(left))
    }

    pub fn clamp_abs_top(&self, top: i32) -> i32 {
        self.pixels.clamp_top(self.top + self.clamp_rel_top(top))
    }

    pub fn clamp_abs_left(&self, left: i32) -> i32 {
        self.pixels
            .clamp_left(self.left + self.clamp_rel_left(left))
    }

    pub fn clamp_abs(&self, top: i32, left: i32) -> (i32, i32) {
        (self.clamp_abs_top(top), self.clamp_abs_left(left))
    }

    pub fn draw_line(
        &mut self,
        context: &mut Context,
        top: i32,
        left: i32,
        line: &Line,
        height: u32,
    ) {
        let mut scaler = line.scaler(context);

        while let Some((advance, glyph, image)) = scaler.next() {
            let Some(image) = image else { continue; };

            self.draw_rect(
                top,
                left + advance as i32,
                glyph.advance as u32,
                height,
                glyph.style.background,
            );
            self.draw_image(
                // We have to render at the baseline!
                top + line.size() as i32,
                left + advance as i32,
                image,
                glyph.style.foreground,
            );
        }
    }

    pub fn draw_rect(&mut self, top: i32, left: i32, width: u32, height: u32, color: Rgba) {
        let (top1, left1) = self.clamp_abs(top, left);
        let (top2, left2) = self.clamp_abs(top + height as i32, left + width as i32);

        for top in top1..top2 {
            for left in left1..left2 {
                let pixel = self.pixels.pixel_mut(top as u32, left as u32).unwrap();
                let Rgb { r, g, b } = color.over(Rgb::new(pixel[0], pixel[1], pixel[2]));
                pixel[0] = r;
                pixel[1] = g;
                pixel[2] = b;
            }
        }
    }

    pub fn draw_image(&mut self, top: i32, left: i32, image: &Image, color: Rgba) {
        debug_assert!(image.content == Content::Mask);

        // Swash image has placement
        let top = top - image.placement.top;
        let left = left + image.placement.left;
        let width = image.placement.width as i32;
        let height = image.placement.height as i32;

        // Clamps
        let (top1, left1) = self.clamp_abs(top, left);
        let (top2, left2) = self.clamp_abs(top + height, left + width);

        for dest_top in top1..top2 {
            for dest_left in left1..left2 {
                let mask = {
                    let top = dest_top - self.top - top;
                    let left = dest_left - self.left - left;
                    let index = left + top * width;
                    *image.data.get(index as usize).unwrap()
                };

                if mask == 0 {
                    continue;
                }

                let dest = self
                    .pixels
                    .pixel_mut(dest_top as u32, dest_left as u32)
                    .unwrap();
                let Rgb { r, g, b } = color
                    .scale_alpha(mask)
                    .over(Rgb::new(dest[0], dest[1], dest[2]));

                dest[0] = r;
                dest[1] = g;
                dest[2] = b;
            }
        }
    }
}
