use graphics::{
    colors::Rgba,
    text::{Context, Font, FontStyle, FontWeight, Fonts, Line, Styles},
    wgpu::atlas::Atlas,
};
use image::{GrayImage, ImageFormat};
use swash::scale::image::Content;

const OUTPUT_FOLDER: &str = "/home/romain/Desktop/atlas/";

fn main() {
    for atlas_width in [64, 128, 256] {
        let mut len = 0;
        let mut atlas = Atlas::new(atlas_width);
        let mut context = Context::new(fonts());
        let font_size = 50;
        let font = context
            .fonts()
            .get((
                context.fonts().get("FiraCode").unwrap().key(),
                FontWeight::Regular,
                FontStyle::Normal,
            ))
            .unwrap()
            .key();

        let mut shaper = Line::shaper(&mut context, font_size);
        shaper.push(
            include_str!("../../virus/src/main.rs"),
            Styles {
                font,
                foreground: Rgba::new(255, 0, 0, 255),
                background: Rgba::new(0, 255, 0, 255),
                underline: false,
                strike: false,
            },
        );
        let line = shaper.line();
        let mut scaler = line.scaler(&mut context);

        while let Some((_, glyph, image)) = scaler.next() {
            if let Some(image) = image {
                let [width, height] = [
                    image.placement.width as usize,
                    image.placement.height as usize,
                ];

                if width == 0 || height == 0 || image.content != Content::Mask {
                    continue;
                }

                let key = (font, glyph.id, font_size);

                if atlas.get(&key).is_none() {
                    len += width * height;
                }

                atlas
                    .set(
                        (font, glyph.id, font_size),
                        [
                            image.placement.width as usize,
                            image.placement.height as usize,
                        ],
                        &image.data,
                    )
                    .unwrap();
            }
        }

        let image = GrayImage::from_raw(
            atlas.width() as u32,
            atlas.height() as u32,
            atlas.data().to_owned(),
        )
        .unwrap();

        image
            .save_with_format(
                format!("{OUTPUT_FOLDER}atlas-{atlas_width}.png"),
                ImageFormat::Png,
            )
            .unwrap();

        let atlas_len = atlas.width() * atlas.height();
        let wasted = (atlas_len - len) as f32 / atlas_len as f32;
        dbg!((atlas_width, len, atlas_len, wasted));
    }
}

fn fonts() -> Fonts {
    use FontStyle::*;
    use FontWeight::*;

    const EMOJI: &str = "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf";
    const FIRA_LIGHT: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Light Nerd Font Complete Mono.ttf";
    const FIRA_REGULAR: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Regular Nerd Font Complete Mono.ttf";
    const FIRA_MEDIUM: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Medium Nerd Font Complete Mono.ttf";
    const FIRA_BOLD: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Bold Nerd Font Complete Mono.ttf";

    let mut fonts = Fonts::new(Font::from_file(EMOJI).unwrap());

    let fira_light = fonts.set(Font::from_file(FIRA_LIGHT).unwrap()).unwrap();
    let fira_regular = fonts.set(Font::from_file(FIRA_REGULAR).unwrap()).unwrap();
    let fira_medium = fonts.set(Font::from_file(FIRA_MEDIUM).unwrap()).unwrap();
    let fira_bold = fonts.set(Font::from_file(FIRA_BOLD).unwrap()).unwrap();

    let fira = fonts.set(String::from("FiraCode")).unwrap();
    fonts.set((fira, Light, Normal, fira_light)).unwrap();
    fonts.set((fira, Regular, Normal, fira_regular)).unwrap();
    fonts.set((fira, Medium, Normal, fira_medium)).unwrap();
    fonts.set((fira, Bold, Normal, fira_bold)).unwrap();

    fonts
}
