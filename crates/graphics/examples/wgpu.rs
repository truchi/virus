//! 😍

#![allow(unused)]

use graphics::{
    colors::Rgba,
    text::{Context, Font, FontStyle, FontWeight, Fonts, Line, Styles},
    wgpu::Graphics,
};
use std::{
    mem::size_of,
    time::{Duration, Instant},
};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBinding,
    BufferBindingType, BufferUsages, Color, ColorTargetState, ComputePipeline,
    ComputePipelineDescriptor, Device, FragmentState, Instance, LoadOp, Operations,
    PipelineLayoutDescriptor, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderStages, Surface,
    SurfaceConfiguration, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState,
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
};

// Is it maxed at 60fps?
const FRAME: Duration = Duration::from_millis(1);

// =================================================================================================

pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(&event_loop)
        .unwrap();

    let mut context = Context::new(fonts());
    let font = context
        .fonts()
        .get((
            context.fonts().get("FiraCode").unwrap().key(),
            FontWeight::Regular,
            FontStyle::Normal,
        ))
        .unwrap()
        .key();

    let mut state = Graphics::new(window);
    let mut start = Instant::now();
    let mut last_redraw = Instant::now();

    let font_size = 100;
    let line_height = (font_size as f32 * 1.25).floor() as u32;
    let lines = {
        let mut lines = vec![];

        for (i, line) in include_str!("./wgpu.rs").lines().enumerate() {
            let mut shaper = Line::shaper(&mut context, font_size);
            shaper.push(
                line,
                Styles {
                    font,
                    foreground: Rgba::new(255, 255, 0, 255),
                    background: Rgba::new(0, 0, 255, 255),
                    underline: false,
                    strike: false,
                },
            );

            lines.push(shaper.line());
        }

        lines
    };

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
        }
        Event::MainEventsCleared => {
            state.window().request_redraw();
        }
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            let now = Instant::now();

            if now - last_redraw > FRAME {
                last_redraw = now;

                let scroll = start.elapsed().as_millis() as i32;
                let scroll = 0;

                for (i, line) in lines.iter().enumerate() {
                    state.add_line(
                        &mut context,
                        i as i32 * line_height as i32 - scroll,
                        0,
                        0,
                        line,
                        line_height,
                    );
                }

                state.update();
                state.render();
            }
        }
        _ => {}
    });
}

// =================================================================================================

fn fonts() -> Fonts {
    use FontStyle::*;
    use FontWeight::*;

    const EMOJI: &str = "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf";

    // const UBUNTU: &str = "/usr/share/fonts/truetype/ubuntu/Ubuntu-B.ttf";

    const FIRA_LIGHT: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Light Nerd Font Complete Mono.ttf";
    const FIRA_REGULAR: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Regular Nerd Font Complete Mono.ttf";
    const FIRA_MEDIUM: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Medium Nerd Font Complete Mono.ttf";
    const FIRA_BOLD: &str = "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Bold Nerd Font Complete Mono.ttf";

    const RECURSIVE_REGULAR: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Regular-1.084.ttf";
    const RECURSIVE_BOLD: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Bold-1.084.ttf";
    const RECURSIVE_ITALIC: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Italic-1.084.ttf";
    const RECURSIVE_BOLD_ITALIC: &str = "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-BoldItalic-1.084.ttf";

    const JETBRAINS_THIN: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Thin.ttf";
    const JETBRAINS_EXTRA_LIGHT : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-ExtraLight.ttf";
    const JETBRAINS_LIGHT: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Light.ttf";
    const JETBRAINS_REGULAR: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Regular.ttf";
    const JETBRAINS_MEDIUM: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Medium.ttf";
    const JETBRAINS_BOLD: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Bold.ttf";
    const JETBRAINS_SEMI_BOLD: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-SemiBold.ttf";
    const JETBRAINS_EXTRA_BOLD: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-ExtraBold.ttf";
    const JETBRAINS_THIN_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-ThinItalic.ttf";
    const JETBRAINS_EXTRA_LIGHT_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-ExtraLightItalic.ttf";
    const JETBRAINS_LIGHT_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-LightItalic.ttf";
    const JETBRAINS_ITALIC: &str =
        "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-Italic.ttf";
    const JETBRAINS_MEDIUM_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-MediumItalic.ttf";
    const JETBRAINS_BOLD_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-BoldItalic.ttf";
    const JETBRAINS_SEMI_BOLD_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-SemiBoldItalic.ttf";
    const JETBRAINS_EXTRA_BOLD_ITALIC : &str = "/home/romain/.local/share/fonts/JetBrainsMono-2.304/fonts/ttf/JetBrainsMono-ExtraBoldItalic.ttf";

    const VICTOR_THIN: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Thin.ttf";
    const VICTOR_EXTRA_LIGHT: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-ExtraLight.ttf";
    const VICTOR_LIGHT: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Light.ttf";
    const VICTOR_REGULAR: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Regular.ttf";
    const VICTOR_MEDIUM: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Medium.ttf";
    const VICTOR_SEMI_BOLD: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-SemiBold.ttf";
    const VICTOR_BOLD: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Bold.ttf";
    const VICTOR_THIN_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-ThinItalic.ttf";
    const VICTOR_EXTRA_LIGHT_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-ExtraLightItalic.ttf";
    const VICTOR_LIGHT_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-LightItalic.ttf";
    const VICTOR_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Italic.ttf";
    const VICTOR_MEDIUM_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-MediumItalic.ttf";
    const VICTOR_SEMI_BOLD_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-SemiBoldItalic.ttf";
    const VICTOR_BOLD_ITALIC: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-BoldItalic.ttf";
    const VICTOR_THIN_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-ThinOblique.ttf";
    const VICTOR_EXTRA_LIGHT_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-ExtraLightOblique.ttf";
    const VICTOR_LIGHT_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-LightOblique.ttf";
    const VICTOR_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-Oblique.ttf";
    const VICTOR_MEDIUM_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-MediumOblique.ttf";
    const VICTOR_SEMI_BOLD_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-SemiBoldOblique.ttf";
    const VICTOR_BOLD_OBLIQUE: &str =
        "/home/romain/.local/share/fonts/VictorMonoAll/TTF/VictorMono-BoldOblique.ttf";

    let mut fonts = Fonts::new(Font::from_file(EMOJI).unwrap());

    // let ubuntu = fonts.set(Font::from_file(UBUNTU).unwrap());

    let fira_light = fonts.set(Font::from_file(FIRA_LIGHT).unwrap()).unwrap();
    let fira_regular = fonts.set(Font::from_file(FIRA_REGULAR).unwrap()).unwrap();
    let fira_medium = fonts.set(Font::from_file(FIRA_MEDIUM).unwrap()).unwrap();
    let fira_bold = fonts.set(Font::from_file(FIRA_BOLD).unwrap()).unwrap();

    let recursive_regular = fonts
        .set(Font::from_file(RECURSIVE_REGULAR).unwrap())
        .unwrap();
    let recursive_bold = fonts.set(Font::from_file(RECURSIVE_BOLD).unwrap()).unwrap();
    let recursive_italic = fonts
        .set(Font::from_file(RECURSIVE_ITALIC).unwrap())
        .unwrap();
    let recursive_bold_italic = fonts
        .set(Font::from_file(RECURSIVE_BOLD_ITALIC).unwrap())
        .unwrap();

    let jetbrains_thin = fonts.set(Font::from_file(JETBRAINS_THIN).unwrap()).unwrap();
    let jetbrains_extra_light = fonts
        .set(Font::from_file(JETBRAINS_EXTRA_LIGHT).unwrap())
        .unwrap();
    let jetbrains_light = fonts
        .set(Font::from_file(JETBRAINS_LIGHT).unwrap())
        .unwrap();
    let jetbrains_regular = fonts
        .set(Font::from_file(JETBRAINS_REGULAR).unwrap())
        .unwrap();
    let jetbrains_medium = fonts
        .set(Font::from_file(JETBRAINS_MEDIUM).unwrap())
        .unwrap();
    let jetbrains_bold = fonts.set(Font::from_file(JETBRAINS_BOLD).unwrap()).unwrap();
    let jetbrains_semi_bold = fonts
        .set(Font::from_file(JETBRAINS_SEMI_BOLD).unwrap())
        .unwrap();
    let jetbrains_extra_bold = fonts
        .set(Font::from_file(JETBRAINS_EXTRA_BOLD).unwrap())
        .unwrap();
    let jetbrains_thin_italic = fonts
        .set(Font::from_file(JETBRAINS_THIN_ITALIC).unwrap())
        .unwrap();
    let jetbrains_extra_light_italic = fonts
        .set(Font::from_file(JETBRAINS_EXTRA_LIGHT_ITALIC).unwrap())
        .unwrap();
    let jetbrains_light_italic = fonts
        .set(Font::from_file(JETBRAINS_LIGHT_ITALIC).unwrap())
        .unwrap();
    let jetbrains_italic = fonts
        .set(Font::from_file(JETBRAINS_ITALIC).unwrap())
        .unwrap();
    let jetbrains_medium_italic = fonts
        .set(Font::from_file(JETBRAINS_MEDIUM_ITALIC).unwrap())
        .unwrap();
    let jetbrains_bold_italic = fonts
        .set(Font::from_file(JETBRAINS_BOLD_ITALIC).unwrap())
        .unwrap();
    let jetbrains_semi_bold_italic = fonts
        .set(Font::from_file(JETBRAINS_SEMI_BOLD_ITALIC).unwrap())
        .unwrap();
    let jetbrains_extra_bold_italic = fonts
        .set(Font::from_file(JETBRAINS_EXTRA_BOLD_ITALIC).unwrap())
        .unwrap();

    let victor_thin = fonts.set(Font::from_file(VICTOR_THIN).unwrap()).unwrap();
    let victor_extra_light = fonts
        .set(Font::from_file(VICTOR_EXTRA_LIGHT).unwrap())
        .unwrap();
    let victor_light = fonts.set(Font::from_file(VICTOR_LIGHT).unwrap()).unwrap();
    let victor_regular = fonts.set(Font::from_file(VICTOR_REGULAR).unwrap()).unwrap();
    let victor_medium = fonts.set(Font::from_file(VICTOR_MEDIUM).unwrap()).unwrap();
    let victor_semi_bold = fonts
        .set(Font::from_file(VICTOR_SEMI_BOLD).unwrap())
        .unwrap();
    let victor_bold = fonts.set(Font::from_file(VICTOR_BOLD).unwrap()).unwrap();
    let victor_thin_italic = fonts
        .set(Font::from_file(VICTOR_THIN_ITALIC).unwrap())
        .unwrap();
    let victor_extra_light_italic = fonts
        .set(Font::from_file(VICTOR_EXTRA_LIGHT_ITALIC).unwrap())
        .unwrap();
    let victor_light_italic = fonts
        .set(Font::from_file(VICTOR_LIGHT_ITALIC).unwrap())
        .unwrap();
    let victor_italic = fonts.set(Font::from_file(VICTOR_ITALIC).unwrap()).unwrap();
    let victor_medium_italic = fonts
        .set(Font::from_file(VICTOR_MEDIUM_ITALIC).unwrap())
        .unwrap();
    let victor_semi_bold_italic = fonts
        .set(Font::from_file(VICTOR_SEMI_BOLD_ITALIC).unwrap())
        .unwrap();
    let victor_bold_italic = fonts
        .set(Font::from_file(VICTOR_BOLD_ITALIC).unwrap())
        .unwrap();
    let victor_thin_oblique = fonts
        .set(Font::from_file(VICTOR_THIN_OBLIQUE).unwrap())
        .unwrap();
    let victor_extra_light_oblique = fonts
        .set(Font::from_file(VICTOR_EXTRA_LIGHT_OBLIQUE).unwrap())
        .unwrap();
    let victor_light_oblique = fonts
        .set(Font::from_file(VICTOR_LIGHT_OBLIQUE).unwrap())
        .unwrap();
    let victor_oblique = fonts.set(Font::from_file(VICTOR_OBLIQUE).unwrap()).unwrap();
    let victor_medium_oblique = fonts
        .set(Font::from_file(VICTOR_MEDIUM_OBLIQUE).unwrap())
        .unwrap();
    let victor_semi_bold_oblique = fonts
        .set(Font::from_file(VICTOR_SEMI_BOLD_OBLIQUE).unwrap())
        .unwrap();
    let victor_bold_oblique = fonts
        .set(Font::from_file(VICTOR_BOLD_OBLIQUE).unwrap())
        .unwrap();

    let fira = fonts.set(String::from("FiraCode")).unwrap();
    fonts.set((fira, Light, Normal, fira_light)).unwrap();
    fonts.set((fira, Regular, Normal, fira_regular)).unwrap();
    fonts.set((fira, Medium, Normal, fira_medium)).unwrap();
    fonts.set((fira, Bold, Normal, fira_bold)).unwrap();

    let recursive = fonts.set(String::from("Recursive")).unwrap();
    fonts
        .set((recursive, Regular, Normal, recursive_regular))
        .unwrap();
    fonts
        .set((recursive, Bold, Normal, recursive_bold))
        .unwrap();
    fonts
        .set((recursive, Regular, Italic, recursive_italic))
        .unwrap();
    fonts
        .set((recursive, Bold, Italic, recursive_bold_italic))
        .unwrap();

    let jetbrains = fonts.set(String::from("JetBrains")).unwrap();
    fonts
        .set((jetbrains, Thin, Normal, jetbrains_thin))
        .unwrap();
    fonts
        .set((jetbrains, ExtraLight, Normal, jetbrains_extra_light))
        .unwrap();
    fonts
        .set((jetbrains, Light, Normal, jetbrains_light))
        .unwrap();
    fonts
        .set((jetbrains, Regular, Normal, jetbrains_regular))
        .unwrap();
    fonts
        .set((jetbrains, Medium, Normal, jetbrains_medium))
        .unwrap();
    fonts
        .set((jetbrains, SemiBold, Normal, jetbrains_semi_bold))
        .unwrap();
    fonts
        .set((jetbrains, Bold, Normal, jetbrains_bold))
        .unwrap();
    fonts
        .set((jetbrains, ExtraBold, Normal, jetbrains_extra_bold))
        .unwrap();
    fonts
        .set((jetbrains, Thin, Italic, jetbrains_thin_italic))
        .unwrap();
    fonts
        .set((jetbrains, ExtraLight, Italic, jetbrains_extra_light_italic))
        .unwrap();
    fonts
        .set((jetbrains, Light, Italic, jetbrains_light_italic))
        .unwrap();
    fonts
        .set((jetbrains, Regular, Italic, jetbrains_italic))
        .unwrap();
    fonts
        .set((jetbrains, Medium, Italic, jetbrains_medium_italic))
        .unwrap();
    fonts
        .set((jetbrains, SemiBold, Italic, jetbrains_semi_bold_italic))
        .unwrap();
    fonts
        .set((jetbrains, Bold, Italic, jetbrains_bold_italic))
        .unwrap();
    fonts
        .set((jetbrains, ExtraBold, Italic, jetbrains_extra_bold_italic))
        .unwrap();

    let victor = fonts.set(String::from("Victor")).unwrap();
    fonts.set((victor, Thin, Normal, victor_thin)).unwrap();
    fonts
        .set((victor, ExtraLight, Normal, victor_extra_light))
        .unwrap();
    fonts.set((victor, Light, Normal, victor_light)).unwrap();
    fonts
        .set((victor, Regular, Normal, victor_regular))
        .unwrap();
    fonts.set((victor, Medium, Normal, victor_medium)).unwrap();
    fonts
        .set((victor, SemiBold, Normal, victor_semi_bold))
        .unwrap();
    fonts.set((victor, Bold, Normal, victor_bold)).unwrap();
    fonts
        .set((victor, Thin, Italic, victor_thin_italic))
        .unwrap();
    fonts
        .set((victor, ExtraLight, Italic, victor_extra_light_italic))
        .unwrap();
    fonts
        .set((victor, Light, Italic, victor_light_italic))
        .unwrap();
    fonts.set((victor, Regular, Italic, victor_italic)).unwrap();
    fonts
        .set((victor, Medium, Italic, victor_medium_italic))
        .unwrap();
    fonts
        .set((victor, SemiBold, Italic, victor_semi_bold_italic))
        .unwrap();
    fonts
        .set((victor, Bold, Italic, victor_bold_italic))
        .unwrap();
    fonts
        .set((victor, Thin, Oblique, victor_thin_oblique))
        .unwrap();
    fonts
        .set((victor, ExtraLight, Oblique, victor_extra_light_oblique))
        .unwrap();
    fonts
        .set((victor, Light, Oblique, victor_light_oblique))
        .unwrap();
    fonts
        .set((victor, Regular, Oblique, victor_oblique))
        .unwrap();
    fonts
        .set((victor, Medium, Oblique, victor_medium_oblique))
        .unwrap();
    fonts
        .set((victor, SemiBold, Oblique, victor_semi_bold_oblique))
        .unwrap();
    fonts
        .set((victor, Bold, Oblique, victor_bold_oblique))
        .unwrap();

    fonts
}
