#![allow(unused)]

use virus_common::{Rectangle, Rgb, Rgba};
use virus_graphics::{
    text::{
        Context, FontFamilyKey, FontKey, FontSize, FontStyle, FontWeight, Line, LineHeight, Styles,
    },
    wgpu::{Draw, Layer},
};

const MIN_WIDTH: f32 = 0.5;

fn min_width(width: u32) -> u32 {
    (width as f32 * MIN_WIDTH).round() as u32
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           FilesView                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct FilesView {
    is_hidden: bool,
    search: String,
    opens: Vec<String>,
    files: Vec<String>,
    family: FontFamilyKey,
    font_size: FontSize,
    line_height: LineHeight,
    background: Rgba,
}

impl FilesView {
    pub fn new(
        family: FontFamilyKey,
        font_size: FontSize,
        line_height: LineHeight,
        background: Rgba,
    ) -> Self {
        Self {
            is_hidden: true,
            search: Default::default(),
            opens: Default::default(),
            files: Default::default(),
            family,
            font_size,
            line_height,
            background,
        }
    }

    pub fn render(&mut self, context: &mut Context, layer: &mut Layer) {
        let mut renderer = Renderer::new(
            context,
            layer,
            self.background,
            self.family,
            self.font_size,
            self.line_height,
            [0, 0],
            [10, 100],
        );

        renderer.render_background();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Renderer                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Renderer<'context, 'layer, 'graphics> {
    context: &'context mut Context,
    layer: &'layer mut Layer<'graphics>,
    background: Rgba,
    family: FontFamilyKey,
    font_size: FontSize,
    line_height: LineHeight,
    advance: f32,
    top: u32,
    left: u32,
    rows: u32,
    columns: u32,
}

impl<'context, 'layer, 'graphics> Renderer<'context, 'layer, 'graphics> {
    fn new(
        context: &'context mut Context,
        layer: &'layer mut Layer<'graphics>,
        background: Rgba,
        family: FontFamilyKey,
        font_size: FontSize,
        line_height: LineHeight,
        [top, left]: [u32; 2],
        [rows, columns]: [u32; 2],
    ) -> Self {
        let advance = context
            .fonts()
            .get(
                context
                    .fonts()
                    .get(family)
                    .unwrap()
                    .best_match_regular_normal()
                    .unwrap(),
            )
            .unwrap()
            .advance_for_size(font_size);

        Self {
            context,
            layer,
            background,
            family,
            font_size,
            line_height,
            advance,
            top,
            left,
            rows,
            columns,
        }
    }

    fn render_background(&mut self) {
        self.layer.draw(0).rectangle(
            Rectangle {
                top: self.top as i32,
                left: self.left as i32,
                width: (self.columns as f32 * self.advance).round() as u32,
                height: self.rows * self.line_height,
            },
            // self.background,
            Rgba::RED,
        );
    }

    fn render_search(&mut self, search: &str) {}
}

// // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
// //                                            Renderer                                            //
// // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// struct Renderer2<'a, 'b, 'c, 'd> {
//     view: &'a mut FilesView,
//     context: &'b mut Context,
//     draw: &'c mut Draw<'d>,
// }

// impl<'a, 'b, 'c, 'd> Renderer2<'a, 'b, 'c, 'd> {
//     fn new(view: &'a mut FilesView, context: &'b mut Context, draw: &'c mut Draw<'d>) -> Self {
//         Self {
//             view,
//             context,
//             draw,
//         }
//     }

//     fn render(&mut self) {
//         self.render_background();
//         self.render_search();
//     }
// }

// impl<'a, 'b, 'c, 'd, 'e> Renderer2<'a, 'b, 'c, 'd> {
//     fn render_background(&mut self) {}

//     fn render_search(&mut self) {
//         self.view.search = String::from("SALUT BANDE DE SALOPE");

//         let line = {
//             let mut shaper = Line::shaper(self.context, self.view.font_size);
//             shaper.push(
//                 &self.view.search,
//                 self.view.family,
//                 Styles {
//                     weight: FontWeight::Regular,
//                     style: FontStyle::Normal,
//                     foreground: Rgba::WHITE,
//                     background: Rgba::TRANSPARENT,
//                     underline: false,
//                     strike: false,
//                     shadow: None,
//                 },
//             );
//             shaper.line()
//         };

//         // TODO align horizontally on grid
//         let width = (self.draw.width() as f32 * MIN_WIDTH).round() as u32;
//         let height = self.draw.height();
//         let top = 0;
//         let left = ((self.draw.width() - width) / 2) as i32;

//         self.draw.draw(([top, left], [width, height])).glyphs(
//             self.context,
//             [0, 0],
//             &line,
//             self.view.line_height,
//             std::time::Duration::ZERO,
//         );
//     }
// }
