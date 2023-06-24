use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Constants                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct Constants {
    pub surface: [f32; 2],
    pub texture: [f32; 2],
}

impl Constants {
    // /!\
    pub const SIZE: u32 = 4 * size_of::<f32>() as u32;

    pub fn as_array(&self) -> [f32; 4] {
        [
            self.surface[0],
            self.surface[1],
            self.texture[0],
            self.texture[1],
        ]
    }
}
