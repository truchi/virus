use crate::text::{Context, FontSize, Line};
use swash::{
    scale::image::{Content, Image},
    CacheKey,
};
use virus_common::{Rgb, Rgba, Style};

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

    pub fn rgba(&mut self, top: u32, left: u32) -> Option<&mut [u8]> {
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

    pub fn draw_line(
        &mut self,
        context: &mut Context,
        top: i32,
        left: i32,
        line: &Line,
        size: FontSize,
    ) {
        let mut scaler = line.scaler(context);

        while let Some((advance, glyph, image)) = scaler.next() {
            let Some(image) = image else { continue; };

            self.draw_image(
                // We have to render at the baseline!
                top + size as i32,
                left + advance as i32,
                image,
                glyph.style,
            );
        }
    }

    pub fn draw_image(&mut self, top: i32, left: i32, image: &Image, style: Style) {
        debug_assert!(image.content == Content::Mask);

        // Swash image has placement
        let top = top - image.placement.top;
        let left = left + image.placement.left;
        let width = image.placement.width as i32;
        let height = image.placement.height as i32;

        // Clamp in `Surface`
        let (rel_top1, rel_top2) = (top, top + height);
        let (rel_left1, rel_left2) = (left, left + width);
        let (rel_top1, rel_top2) = (
            clamp(rel_top1, 0, self.height as i32),
            clamp(rel_top2, 0, self.height as i32),
        );
        let (rel_left1, rel_left2) = (
            clamp(rel_left1, 0, self.width as i32),
            clamp(rel_left2, 0, self.width as i32),
        );

        // Clamp in `PixelsMut`
        let (abs_top1, abs_top2) = (self.top + rel_top1, self.top + rel_top2);
        let (abs_left1, abs_left2) = (self.left + rel_left1, self.left + rel_left2);
        let (abs_top1, abs_top2) = (
            clamp(abs_top1, 0, self.pixels.height as i32),
            clamp(abs_top2, 0, self.pixels.height as i32),
        );
        let (abs_left1, abs_left2) = (
            clamp(abs_left1, 0, self.pixels.width as i32),
            clamp(abs_left2, 0, self.pixels.width as i32),
        );

        for dest_top in abs_top1..abs_top2 {
            for dest_left in abs_left1..abs_left2 {
                let mask = {
                    let top = dest_top - self.top - top;
                    let left = dest_left - self.left - left;
                    let index = left + top * width;
                    *image.data.get(index as usize).unwrap()
                };

                if mask == 0 {
                    continue;
                }

                let dest = self.pixels.rgba(dest_top as u32, dest_left as u32).unwrap();
                let Rgba { r, g, b, a } = style
                    .foreground
                    .scale_alpha(mask)
                    .over(Rgba::new(dest[0], dest[1], dest[2], dest[3]));

                dest[0] = r;
                dest[1] = g;
                dest[2] = b;
                dest[3] = a;
            }
        }
    }
}

fn clamp(i: i32, min: i32, max: i32) -> i32 {
    i.max(min).min(max)
}
