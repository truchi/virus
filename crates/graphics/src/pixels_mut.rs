#[derive(Debug)]
pub struct PixelsMut<'a> {
    width: u32,
    height: u32,
    pixels: &'a mut [u8],
}

impl<'a> PixelsMut<'a> {
    pub fn new(width: u32, height: u32, pixels: &'a mut [u8]) -> Self {
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

    pub fn draw(&self) {}
}
