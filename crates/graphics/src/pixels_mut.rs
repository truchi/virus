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
        let index = left as usize + top as usize * self.width as usize;

        self.pixels.get_mut(4 * index..4 * (index + 1))
    }

    pub fn over(&mut self, top: u32, left: u32, color: Rgba) {
        if top >= self.height || left >= self.width {
            return;
        }

        let pixel = {
            let index = left as usize + top as usize * self.width as usize;
            self.pixels.get_mut(4 * index..4 * (index + 1)).unwrap()
        };

        let Rgb { r, g, b } = color.over(Rgb::new(pixel[0], pixel[1], pixel[2]));
        pixel[0] = r;
        pixel[1] = g;
        pixel[2] = b;
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

            let baseline = top + line.size() as i32;
            let left = left + advance as i32;
            let (foreground, background) = (glyph.style.foreground, glyph.style.background);

            self.draw_rect(top, left, glyph.advance as u32, height, background);
            self.draw_image(baseline, left, image, foreground);
        }
    }

    pub fn draw_rect(&mut self, top: i32, left: i32, width: u32, height: u32, color: Rgba) {
        let (top1, left1) = self.clamp_abs(top, left);
        let (top2, left2) = self.clamp_abs(top + height as i32, left + width as i32);

        for top in top1..top2 {
            for left in left1..left2 {
                self.pixels.over(top as u32, left as u32, color);
            }
        }
    }

    pub fn draw_image(&mut self, top: i32, left: i32, image: &Image, color: Rgba) {
        match image.content {
            Content::Mask => self.draw_image_mask(top, left, image, color),
            Content::Color => self.draw_image_color(top, left, image),
            Content::SubpixelMask => unreachable!(),
        }
    }

    pub fn draw_image_mask(&mut self, top: i32, left: i32, image: &Image, color: Rgba) {
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

                self.pixels
                    .over(dest_top as u32, dest_left as u32, color.scale_alpha(mask));
            }
        }
    }

    pub fn draw_image_color(&mut self, top: i32, left: i32, image: &Image) {
        debug_assert!(image.content == Content::Color);

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
                let src = {
                    let top = dest_top - self.top - top;
                    let left = dest_left - self.left - left;
                    let index = (left + top * width) as usize;
                    let pixel = image.data.get(4 * index..4 * (index + 1)).unwrap();
                    Rgba::new(pixel[0], pixel[1], pixel[2], pixel[3])
                };

                self.pixels.over(dest_top as u32, dest_left as u32, src);
            }
        }
    }

    pub fn draw_circle(&mut self, top: i32, left: i32, radius: u32, color: Rgba) {
        let (center_top, center_left) = (top, left);
        let rel_widths = 0..self.width as i32;
        let rel_heights = 0..self.height as i32;
        let abs_widths = 0..self.pixels.width as i32;
        let abs_heights = 0..self.pixels.height as i32;

        let center = |(top, left)| (center_top + top, center_left + left);
        let position = |(top, left)| (self.top + top, self.left + left);
        let clamp_rel =
            |(top, left): &(i32, i32)| rel_heights.contains(&top) && rel_widths.contains(&left);
        let clamp_abs =
            |(top, left): &(i32, i32)| abs_heights.contains(&top) && abs_widths.contains(&left);

        for (top, left) in full(bresenham(radius))
            .map(center)
            .filter(clamp_rel)
            .map(position)
            .filter(clamp_abs)
        {
            self.pixels.over(top as u32, left as u32, color);
        }
    }

    pub fn draw_circle_thick(&mut self, top: i32, left: i32, radius: u32, width: u32, color: Rgba) {
        let (center_top, center_left) = (top, left);
        let rel_widths = 0..self.width as i32;
        let rel_heights = 0..self.height as i32;
        let abs_widths = 0..self.pixels.width as i32;
        let abs_heights = 0..self.pixels.height as i32;

        let center = |(top, left)| (center_top + top, center_left + left);
        let position = |(top, left)| (self.top + top, self.left + left);
        let clamp_rel =
            |(top, left): &(i32, i32)| rel_heights.contains(&top) && rel_widths.contains(&left);
        let clamp_abs =
            |(top, left): &(i32, i32)| abs_heights.contains(&top) && abs_widths.contains(&left);

        for (top, left) in full(bresenham_thick(radius, radius + width))
            .map(center)
            .filter(clamp_rel)
            .map(position)
            .filter(clamp_abs)
        {
            self.pixels.over(top as u32, left as u32, color);
        }
    }
}

fn bresenham(radius: u32) -> impl Iterator<Item = (i32, i32)> {
    let r = radius as i32;

    let mut x = r;
    let mut y = 0;
    let mut e = -r;

    std::iter::from_fn(move || {
        if y > x {
            None
        } else {
            let top_left = (-y, x);

            e += 2 * y + 1;
            y += 1;

            if e >= 0 {
                e -= 2 * x - 1;
                x -= 1;
            }

            Some(top_left)
        }
    })
}

fn bresenham_thick(inner: u32, outer: u32) -> impl Iterator<Item = (i32, i32)> {
    debug_assert!(inner <= outer);

    let mut inners = bresenham(inner);
    let mut outers = bresenham(outer);
    let mut last_inner_left = 0;

    std::iter::from_fn(move || match (inners.next(), outers.next()) {
        (Some((inner_top, inner_left)), Some((outer_top, outer_left))) => {
            debug_assert!(inner_top == outer_top);
            debug_assert!(inner_left <= outer_left);

            last_inner_left = inner_left;
            Some((outer_top, inner_left..outer_left))
        }
        (None, Some((outer_top, outer_left))) => Some((outer_top, last_inner_left..outer_left)),
        (Some(_), None) => unreachable!(),
        _ => None,
    })
    .flat_map(|(top, lefts)| lefts.map(move |left| (top, left)))
}

fn full<T: Iterator<Item = (i32, i32)>>(it: T) -> impl Iterator<Item = (i32, i32)> {
    it.flat_map(|(top, left)| {
        [
            (top, left),
            (top, -left),
            (-top, left),
            (-top, -left),
            (left, top),
            (left, -top),
            (-left, top),
            (-left, -top),
        ]
    })
}
