use crate::events::{Event, Events};
use pixels::{Pixels, SurfaceTexture};
use virus_editor::{Document, Language, Theme};
use virus_graphics::{
    pixels_mut::PixelsMut,
    text::{Context, Font, Fonts},
};
use virus_ui::document_view::DocumentView;
use winit::{
    dpi::PhysicalSize,
    event::VirtualKeyCode,
    event_loop::EventLoop,
    window::{Fullscreen, Window, WindowBuilder},
};

const RECURSIVE_VF: &str =
    "/home/romain/.local/share/fonts/Recursive/Recursive_Desktop/Recursive_VF_1.084.ttf";
const RECURSIVE: &str =
    "/home/romain/.local/share/fonts/Recursive/Recursive_Code/RecMonoDuotone/RecMonoDuotone-Regular-1.084.ttf";
const UBUNTU: &str = "/usr/share/fonts/truetype/ubuntu/Ubuntu-B.ttf";
const FIRA: &str =
    "/home/romain/.local/share/fonts/FiraCodeNerdFont/Fira Code Regular Nerd Font Complete Mono.ttf";
const EMOJI: &str = "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf";

const HIGHLIGHT_QUERY: &str = include_str!("../../editor/treesitter/rust/highlights.scm");

const SCALE: u32 = 1;

pub struct Virus {
    window: Window,
    event_loop: EventLoop<()>,
    events: Events,
    context: Context,
    document: Document,
    document_view: DocumentView,
    pixels: Pixels,
}

impl Virus {
    pub fn new() -> Self {
        let fira = Font::from_file(FIRA).unwrap();
        let ubuntu = Font::from_file(UBUNTU).unwrap();
        let recursive = Font::from_file(RECURSIVE).unwrap();
        let emoji = Font::from_file(EMOJI).unwrap();

        let font = recursive;
        let key = font.key();
        let mut context = Context::new(Fonts::new([font], emoji));

        let event_loop = EventLoop::new();
        let window = {
            let window = WindowBuilder::new()
                .with_title("virus")
                .with_inner_size(PhysicalSize::new(1, 1))
                .with_fullscreen(Some(Fullscreen::Borderless(None)))
                .build(&event_loop)
                .unwrap();
            window.set_cursor_visible(false);
            window
        };
        let mut events = Events::new(window.id());

        let mut pixels = {
            let PhysicalSize { width, height } = window.inner_size();
            Pixels::new(width, height, SurfaceTexture::new(width, height, &window)).unwrap()
        };

        let mut document = Document::empty(Some(Language::Rust));
        document.parse();
        let mut document_view =
            DocumentView::new(HIGHLIGHT_QUERY.into(), Theme::dracula(), key, 40, 50);

        Self {
            window,
            event_loop,
            events,
            context,
            document,
            document_view,
            pixels,
        }
    }

    pub fn run(mut self) {
        self.event_loop.run(move |event, _, control_flow| {
            let event = match self.events.update(&event) {
                Some(event) => event,
                None => return,
            };

            // dbg!(event);
            match event {
                Event::Char(char) => {
                    self.document.edit_char(char);
                    self.document.parse();
                }
                Event::Pressed(VirtualKeyCode::Escape) => {
                    control_flow.set_exit();
                }
                Event::Pressed(_) => {}
                Event::Released(_) => {}
                Event::Resized(PhysicalSize { width, height }) => {
                    if width != 1 {
                        self.pixels.resize_surface(width, height).unwrap();
                        self.pixels
                            .resize_buffer(width / SCALE, height / SCALE)
                            .unwrap();
                    }
                }
                Event::Moved(_) => {}
                Event::Focused => {}
                Event::Unfocused => {}
                Event::Close => {}
                Event::Closed => {}
                Event::Update => {
                    self.window.request_redraw();
                }
                Event::Redraw => {
                    let mut pixels_mut = {
                        let PhysicalSize { width, height } = self.window.inner_size();
                        PixelsMut::new(width / SCALE, height / SCALE, self.pixels.get_frame_mut())
                    };

                    for (i, u) in pixels_mut.pixels_mut().iter_mut().enumerate() {
                        *u = match i % 4 {
                            0 => 0,
                            1 => 0,
                            2 => 0,
                            _ => 255,
                        };
                    }

                    let width = pixels_mut.width();
                    let height = pixels_mut.height();

                    if pixels_mut.pixels().len() == 4 {
                        return;
                    }

                    self.document_view.render(
                        &mut pixels_mut.surface(0, 0, width, height),
                        &mut self.context,
                        &self.document,
                        0,
                    );

                    self.pixels.render().unwrap();
                }
                Event::Quit => {}
            }
        });
    }
}
