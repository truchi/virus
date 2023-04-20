//! Winit events helper.

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        ElementState, Event as WinitEvent, KeyboardInput, ModifiersState, VirtualKeyCode,
        WindowEvent,
    },
    window::WindowId,
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
    Close,
    Closed,
    Update,
    Redraw,
    Quit,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Events                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct Events {
    window_id: WindowId,
    modifiers: ModifiersState,
}

impl Events {
    pub fn new(window_id: WindowId) -> Self {
        Self {
            window_id,
            modifiers: Default::default(),
        }
    }

    pub fn update<T>(&mut self, event: &WinitEvent<T>) -> Option<Event> {
        match event {
            WinitEvent::NewEvents(_) => None,
            WinitEvent::WindowEvent { window_id, event } => {
                if self.window_id == *window_id {
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
            WinitEvent::MainEventsCleared => Some(Event::Update),
            WinitEvent::RedrawRequested(_) => Some(Event::Redraw),
            WinitEvent::RedrawEventsCleared => None,
            WinitEvent::LoopDestroyed => Some(Event::Quit),
        }
    }
}
