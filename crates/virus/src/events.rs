//! Winit events helper.

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        ElementState, Event as WinitEvent, KeyboardInput, ModifiersState, VirtualKeyCode,
        WindowEvent,
    },
    window::{Window, WindowId},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Event                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub enum Event {
    Char(char),
    Pressed(VirtualKeyCode),
    Released(VirtualKeyCode),
    Resized(PhysicalSize<u32>),
    Moved(PhysicalPosition<i32>),
    Focused,
    Unfocused,
    Redraw,
    Close,
    Closed,
    Quit,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Events                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

const DEBUG: bool = false;

#[derive(Debug)]
pub struct Events {
    modifiers: ModifiersState,
}

impl Events {
    pub fn new() -> Self {
        Self {
            modifiers: Default::default(),
        }
    }

    pub fn modifiers(&self) -> ModifiersState {
        self.modifiers
    }

    pub fn update<T: std::fmt::Debug>(
        &mut self,
        winit_event: &WinitEvent<T>,
        window: &Window,
    ) -> Option<Event> {
        if DEBUG {
            match winit_event {
                WinitEvent::NewEvents(_) => {
                    println!(
                        "======================================================================="
                    );
                }
                _ => {}
            }
        }

        let event = match winit_event {
            WinitEvent::NewEvents(_) => None,
            WinitEvent::WindowEvent { window_id, event } => {
                if window.id() == *window_id {
                    match event {
                        WindowEvent::Resized(size) => Some(Event::Resized(*size)),
                        WindowEvent::Moved(position) => Some(Event::Moved(*position)),
                        WindowEvent::CloseRequested => Some(Event::Close),
                        WindowEvent::Destroyed => Some(Event::Closed),
                        WindowEvent::DroppedFile(_) => None,
                        WindowEvent::HoveredFile(_) => None,
                        WindowEvent::HoveredFileCancelled => None,
                        WindowEvent::ReceivedCharacter(char) => Some(Event::Char(*char)),
                        WindowEvent::Focused(true) => Some(Event::Focused),
                        WindowEvent::Focused(false) => Some(Event::Unfocused),
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: None,
                                    ..
                                },
                            ..
                        } => None,
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(keycode),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        } => Some(Event::Pressed(*keycode)),
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(keycode),
                                    state: ElementState::Released,
                                    ..
                                },
                            ..
                        } => Some(Event::Released(*keycode)),
                        WindowEvent::ModifiersChanged(modifiers) => {
                            self.modifiers = *modifiers;
                            None
                        }
                        WindowEvent::Ime(_) => None,
                        WindowEvent::CursorMoved { .. } => None,
                        WindowEvent::CursorEntered { .. } => None,
                        WindowEvent::CursorLeft { .. } => None,
                        WindowEvent::MouseWheel { .. } => None,
                        WindowEvent::MouseInput { .. } => None,
                        WindowEvent::TouchpadMagnify { .. } => None,
                        WindowEvent::SmartMagnify { .. } => None,
                        WindowEvent::TouchpadRotate { .. } => None,
                        WindowEvent::TouchpadPressure { .. } => None,
                        WindowEvent::AxisMotion { .. } => None,
                        WindowEvent::Touch(_) => None,
                        WindowEvent::ScaleFactorChanged { .. } => None,
                        WindowEvent::ThemeChanged(_) => None,
                        WindowEvent::Occluded(_) => None,
                    }
                } else {
                    None
                }
            }
            WinitEvent::DeviceEvent { .. } => None,
            WinitEvent::UserEvent(_) => None,
            WinitEvent::Suspended => None,
            WinitEvent::Resumed => None,
            WinitEvent::MainEventsCleared => {
                window.request_redraw();
                None
            }
            WinitEvent::RedrawRequested(_) => Some(Event::Redraw),
            WinitEvent::RedrawEventsCleared => None,
            WinitEvent::LoopDestroyed => Some(Event::Quit),
        };

        if DEBUG {
            if let Some(event) = event {
                println!("\x1B[0;32m{event:#?}\x1B[0m");
            } else {
                println!("\x1B[0;31m{winit_event:#?}\x1B[0m");
            }

            match winit_event {
                WinitEvent::RedrawEventsCleared => {
                    println!(
                        "======================================================================="
                    );
                }
                _ => {}
            }
        }

        event
    }
}
