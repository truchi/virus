//! Winit events helper.

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent, Modifiers, WindowEvent},
    keyboard::{Key as LogicalKey, ModifiersKeyState, NamedKey},
    window::{Window, WindowId},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Key                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub enum Key<'event> {
    Str(&'event str),
    Tab,
    Space,
    Backspace,
    Enter,
    Escape,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Event                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub enum Event<'event> {
    Key(Key<'event>),
    Resized,
    Redraw,
    Close,
    Closed,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Events                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

const DEBUG: bool = false;

#[derive(Debug)]
pub struct Events {
    modifiers: Modifiers,
}

impl Events {
    pub fn new() -> Self {
        Self {
            modifiers: Default::default(),
        }
    }

    pub fn shift(&self) -> bool {
        self.left_shift() || self.right_shift()
    }

    pub fn alt(&self) -> bool {
        self.left_alt() || self.right_alt()
    }

    pub fn control(&self) -> bool {
        self.left_control() || self.right_control()
    }

    pub fn command(&self) -> bool {
        self.left_command() || self.right_command()
    }

    pub fn update<'event>(&mut self, event: &'event WindowEvent) -> Option<Event<'event>> {
        match event {
            WindowEvent::Resized(_) => Some(Event::Resized),
            WindowEvent::CloseRequested => Some(Event::Close),
            WindowEvent::Destroyed => Some(Event::Closed),
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key, state, ..
                },
                ..
            } if *state == ElementState::Pressed => match logical_key.as_ref() {
                LogicalKey::Named(NamedKey::Tab) => Some(Event::Key(Key::Tab)),
                LogicalKey::Named(NamedKey::Space) => Some(Event::Key(Key::Space)),
                LogicalKey::Named(NamedKey::Backspace) => Some(Event::Key(Key::Backspace)),
                LogicalKey::Named(NamedKey::Enter) => Some(Event::Key(Key::Enter)),
                LogicalKey::Named(NamedKey::Escape) => Some(Event::Key(Key::Escape)),
                LogicalKey::Character(str) => Some(Event::Key(Key::Str(str))),
                _ => None,
            },
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = *modifiers;
                None
            }
            WindowEvent::RedrawRequested => Some(Event::Redraw),
            _ => None,
        }
    }
}

/// Private.
impl Events {
    fn left_shift(&self) -> bool {
        self.modifiers.lshift_state() == ModifiersKeyState::Pressed
    }

    fn right_shift(&self) -> bool {
        self.modifiers.rshift_state() == ModifiersKeyState::Pressed
    }

    fn left_alt(&self) -> bool {
        self.modifiers.lalt_state() == ModifiersKeyState::Pressed
    }

    fn right_alt(&self) -> bool {
        self.modifiers.ralt_state() == ModifiersKeyState::Pressed
    }

    fn left_control(&self) -> bool {
        self.modifiers.lcontrol_state() == ModifiersKeyState::Pressed
    }

    fn right_control(&self) -> bool {
        self.modifiers.rcontrol_state() == ModifiersKeyState::Pressed
    }

    fn left_command(&self) -> bool {
        self.modifiers.lsuper_state() == ModifiersKeyState::Pressed
    }

    fn right_command(&self) -> bool {
        self.modifiers.rsuper_state() == ModifiersKeyState::Pressed
    }
}
