use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Constants                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct Constants {
    pub surface: [f32; 2],
}

impl Constants {
    // /!\
    pub const SIZE: u32 = 2 * size_of::<f32>() as u32;
    pub const STAGES: ShaderStages = ShaderStages::VERTEX_FRAGMENT;

    pub fn as_array(&self) -> [f32; 2] {
        [self.surface[0], self.surface[1]]
    }
}
