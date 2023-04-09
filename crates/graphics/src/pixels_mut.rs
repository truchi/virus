use crate::text::{Context, Line, LineHeight};
use std::ops::Range;
use swash::scale::image::{Content, Image};
use virus_common::{Rgb, Rgba};

#[derive(Copy, Clone, Debug)]
pub enum Quadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Quadrant {
    fn with<T: Iterator<Item = (i32, i32)>>(&self, it: T) -> impl Iterator<Item = (i32, i32)> {
        type F = fn((i32, i32)) -> [(i32, i32); 2];

        fn top_left((top, left): (i32, i32)) -> [(i32, i32); 2] {
            [(top, -left), (-left, top)]
        }
        fn top_right((top, left): (i32, i32)) -> [(i32, i32); 2] {
            [(top, left), (-left, -top)]
        }
        fn bottom_left((top, left): (i32, i32)) -> [(i32, i32); 2] {
            [(left, top), (-top, -left)]
        }
        fn bottom_right((top, left): (i32, i32)) -> [(i32, i32); 2] {
            [(left, -top), (-top, left)]
        }

        match self {
            Self::TopLeft => it.flat_map(top_left as F),
            Self::TopRight => it.flat_map(top_right as F),
            Self::BottomLeft => it.flat_map(bottom_left as F),
            Self::BottomRight => it.flat_map(bottom_right as F),
        }
    }
}

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
        height: LineHeight,
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

    pub fn draw_quadrant(
        &mut self,
        quadrant: Quadrant,
        top: i32,
        left: i32,
        radius: u32,
        color: Rgba,
    ) {
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

        for (top, left) in quadrant
            .with(bresenham(radius))
            .map(center)
            .filter(clamp_rel)
            .map(position)
            .filter(clamp_abs)
        {
            self.pixels.over(top as u32, left as u32, color);
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

    pub fn draw_horizontal_line(
        &mut self,
        top: i32,
        Range { start, end }: Range<i32>,
        color: Rgba,
    ) {
        let start = self.clamp_abs_left(start);
        let end = self.clamp_abs_left(end);

        for left in start..end {
            self.pixels.over(top as u32, left as u32, color);
        }
    }

    pub fn draw_vertical_line(&mut self, Range { start, end }: Range<i32>, left: i32, color: Rgba) {
        let start = self.clamp_abs_top(start);
        let end = self.clamp_abs_top(end);

        for top in start..end {
            self.pixels.over(top as u32, left as u32, color);
        }
    }

    pub fn stroke_rect(
        &mut self,
        top: i32,
        left: i32,
        width: u32,
        height: u32,
        radius: u32,
        color: Rgba,
    ) {
        let width = width as i32;
        let height = height as i32;
        let iradius = radius as i32;
        let right = left + width;
        let bottom = top + height;
        let ctop = top + iradius;
        let cbottom = bottom - iradius;
        let cleft = left + iradius;
        let cright = right - iradius;

        self.draw_horizontal_line(top, cleft..cright, color);
        self.draw_horizontal_line(bottom, cleft..cright, color);
        self.draw_vertical_line(ctop..cbottom, left, color);
        self.draw_vertical_line(ctop..cbottom, right, color);
        self.draw_quadrant(Quadrant::TopLeft, ctop, cleft, radius, color);
        self.draw_quadrant(Quadrant::TopRight, ctop, cright, radius, color);
        self.draw_quadrant(Quadrant::BottomLeft, cbottom, cleft, radius, color);
        self.draw_quadrant(Quadrant::BottomRight, cbottom, cright, radius, color);
    }

    pub fn stroke_corner(
        &mut self,
        quadrant: Quadrant,
        top: i32,
        left: i32,
        width: u32,
        height: u32,
        radius: u32,
        color: Rgba,
    ) {
        debug_assert!(width >= radius);
        debug_assert!(height >= radius);

        match quadrant {
            Quadrant::TopLeft => {
                let bottom = top + height as i32;
                let right = left + width as i32;
                let ctop = top + radius as i32;
                let cleft = left + radius as i32;

                self.draw_quadrant(quadrant, ctop, cleft, radius, color);
                self.draw_vertical_line(ctop..bottom, left, color);
                self.draw_horizontal_line(top, cleft..right, color);
            }
            Quadrant::TopRight => {
                let bottom = top + height as i32;
                let right = left;
                let left = right - width as i32;
                let ctop = top + radius as i32;
                let cright = right - radius as i32;

                self.draw_quadrant(quadrant, ctop, cright, radius, color);
                self.draw_vertical_line(ctop..bottom, right, color);
                self.draw_horizontal_line(top, left..cright, color);
            }
            Quadrant::BottomLeft => {
                let bottom = top;
                let top = bottom - height as i32;
                let right = left + width as i32;
                let cbottom = bottom - radius as i32;
                let cleft = left + radius as i32;

                self.draw_quadrant(quadrant, cbottom, cleft, radius, color);
                self.draw_vertical_line(top..cbottom, left, color);
                self.draw_horizontal_line(bottom, cleft..right, color);
            }
            Quadrant::BottomRight => {
                let bottom = top;
                let top = bottom - height as i32;
                let right = left;
                let left = right - width as i32;
                let cbottom = bottom - radius as i32;
                let cright = right - radius as i32;

                self.draw_quadrant(quadrant, cbottom, cright, radius, color);
                self.draw_vertical_line(top..cbottom, right, color);
                self.draw_horizontal_line(bottom, left..cright, color);
            }
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
