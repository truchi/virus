use super::Fonts;
use swash::{scale::ScaleContext, shape::ShapeContext};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Context                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Context for font shaping and scaling.
pub struct Context {
    /// Font cache.
    fonts: Fonts,
    /// Shape context.
    shape: ShapeContext,
    /// Scale context.
    scale: ScaleContext,
}

impl Context {
    /// Creates a new `Context` with `fonts`.
    pub fn new(fonts: Fonts) -> Self {
        Self {
            fonts,
            shape: Default::default(),
            scale: Default::default(),
        }
    }

    /// Returns the font cache.
    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    /// Returns a tuple of mutable references.
    pub fn as_muts(&mut self) -> (&mut Fonts, &mut ShapeContext, &mut ScaleContext) {
        (&mut self.fonts, &mut self.shape, &mut self.scale)
    }
}
