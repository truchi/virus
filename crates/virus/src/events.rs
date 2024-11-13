//! Winit events helper.

use smol_str::SmolStr;
use std::{fmt::Write, ops::BitOr, str::FromStr};
use winit::{
    event::{ElementState, Modifiers, WindowEvent},
    keyboard::{Key as WinitKey, ModifiersKeyState, NamedKey},
    platform::modifier_supplement::KeyEventExtModifierSupplement,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Mods                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Mods {
    control: bool,
    shift: bool,
    alt: bool,
    command: bool,
}

impl Mods {
    pub const NONE: Self = Self::new(false, false, false, false);
    pub const CONTROL: Self = Self::new(true, false, false, false);
    pub const SHIFT: Self = Self::new(false, true, false, false);
    pub const ALT: Self = Self::new(false, false, true, false);
    pub const COMMAND: Self = Self::new(false, false, false, true);

    pub const CONTROL_STR: &'static str = "control";
    pub const SHIFT_STR: &'static str = "shift";
    pub const ALT_STR: &'static str = "alt";
    pub const COMMAND_STR: &'static str = "command";

    pub const fn new(control: bool, shift: bool, alt: bool, command: bool) -> Self {
        Self {
            control,
            shift,
            alt,
            command,
        }
    }

    pub fn control(&self) -> bool {
        self.control
    }

    pub fn shift(&self) -> bool {
        self.shift
    }

    pub fn alt(&self) -> bool {
        self.alt
    }

    pub fn command(&self) -> bool {
        self.command
    }
}

impl BitOr for Mods {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            control: self.control || rhs.control,
            shift: self.shift || rhs.shift,
            alt: self.alt || rhs.alt,
            command: self.command || rhs.command,
        }
    }
}

impl std::fmt::Debug for Mods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (bool, str) in [
            (self.control, Self::CONTROL_STR),
            (self.shift, Self::SHIFT_STR),
            (self.alt, Self::ALT_STR),
            (self.command, Self::COMMAND_STR),
        ] {
            if bool {
                f.write_str(str)?;
                f.write_char(' ')?;
            }
        }

        Ok(())
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Key                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

macro_rules! key {
    ($($variant:ident $str:literal),* $(,)?) => {
        #[derive(Copy, Clone, Eq, PartialEq, Hash)]
        pub enum Key<T = SmolStr> {
            Str(T),
            $($variant,)*
        }

        impl<T: std::fmt::Debug> std::fmt::Debug for Key<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Key::Str(str) => std::fmt::Debug::fmt(str, f),
                    $(Key::$variant => f.write_str(stringify!($variant)),)*
                }
            }
        }

        impl Key<SmolStr> {
            pub fn as_str(&self) -> Key<&str> {
                match self {
                    Key::Str(smol) => Key::Str(smol.as_str()),
                    $(Key::$variant => Key::$variant,)*
                }
            }
        }

        impl<'a> Key<&'a str> {
            pub fn to_smol(&self) -> Key<SmolStr> {
                match self {
                    Key::Str(str) => Key::Str(SmolStr::new(str)),
                    $(Key::$variant => Key::$variant,)*
                }
            }

            pub fn parse(str: &'a str) -> Self {
                match str {
                    $($str => Self::$variant,)*
                    _ => Self::Str(str),
                }
            }

            pub fn to_str(self) -> &'a str {
                match self {
                    $(Self::$variant => $str,)*
                    Self::Str(str) => str,
                }
            }
        }

        impl<'a> TryFrom<WinitKey<&'a str>> for Key {
            type Error = ();

            fn try_from(key: WinitKey<&'a str>) -> Result<Self, Self::Error> {
                Ok(match key {
                    $(WinitKey::Named(NamedKey::$variant) => Self::$variant,)*
                    WinitKey::Character(str) => Self::Str(SmolStr::new(str)),
                    _ => return Err(()),
                })
            }
        }
    };
}

key!(
    ArrowUp     "arrow_up",
    ArrowDown   "arrow_down",
    ArrowLeft   "arrow_left",
    ArrowRight  "arrow_right",
    Tab         "tab",
    Space       "space",
    Backspace   "backspace",
    Enter       "enter",
    Escape      "escape",
);

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            KeyEvent                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Hash, Debug)]
pub struct KeyEvent<T = SmolStr> {
    mods: Mods,
    modded: Key<T>,
    unmodded: Key<T>,
}

impl<T> KeyEvent<T> {
    pub fn new(mods: Mods, modded: Key<T>, unmodded: Key<T>) -> Self {
        Self {
            mods,
            modded,
            unmodded,
        }
    }

    pub fn control(&self) -> bool {
        self.mods.control
    }

    pub fn shift(&self) -> bool {
        self.mods.shift
    }

    pub fn alt(&self) -> bool {
        self.mods.alt
    }

    pub fn command(&self) -> bool {
        self.mods.command
    }

    pub fn modded(&self) -> &Key<T> {
        &self.modded
    }

    pub fn unmodded(&self) -> &Key<T> {
        &self.unmodded
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Event                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Hash, Debug)]
pub enum Event<T = SmolStr> {
    Key(KeyEvent<T>),
    Resized,
    Redraw,
    Close,
    Closed,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Events                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

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

    pub fn update(&mut self, event: &WindowEvent) -> Option<Event> {
        Some(match event {
            WindowEvent::Resized(_) => Event::Resized,
            WindowEvent::CloseRequested => Event::Close,
            WindowEvent::Destroyed => Event::Closed,
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                Event::Key(KeyEvent {
                    mods: Mods::new(self.control(), self.shift(), self.alt(), self.command()),
                    modded: Key::try_from(event.logical_key.as_ref()).ok()?,
                    unmodded: Key::try_from(event.key_without_modifiers().as_ref()).ok()?,
                })
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = *modifiers;
                return None;
            }
            WindowEvent::RedrawRequested => Event::Redraw,
            _ => return None,
        })
    }
}

/// Private.
impl Events {
    fn control(&self) -> bool {
        self.left_control() || self.right_control()
    }

    fn left_control(&self) -> bool {
        self.modifiers.lcontrol_state() == ModifiersKeyState::Pressed
    }

    fn right_control(&self) -> bool {
        self.modifiers.rcontrol_state() == ModifiersKeyState::Pressed
    }

    fn shift(&self) -> bool {
        self.left_shift() || self.right_shift()
    }

    fn left_shift(&self) -> bool {
        self.modifiers.lshift_state() == ModifiersKeyState::Pressed
    }

    fn right_shift(&self) -> bool {
        self.modifiers.rshift_state() == ModifiersKeyState::Pressed
    }

    fn alt(&self) -> bool {
        self.left_alt() || self.right_alt()
    }

    fn left_alt(&self) -> bool {
        self.modifiers.lalt_state() == ModifiersKeyState::Pressed
    }

    fn right_alt(&self) -> bool {
        self.modifiers.ralt_state() == ModifiersKeyState::Pressed
    }

    fn command(&self) -> bool {
        self.left_command() || self.right_command()
    }

    fn left_command(&self) -> bool {
        self.modifiers.lsuper_state() == ModifiersKeyState::Pressed
    }

    fn right_command(&self) -> bool {
        self.modifiers.rsuper_state() == ModifiersKeyState::Pressed
    }
}
