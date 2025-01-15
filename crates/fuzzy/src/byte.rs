// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Class                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum Class {
    /// ASCII uppercase.
    Uppercase,
    /// ASCII lowercase.
    Lowercase,
    /// ASCII digit.
    Digit,
    /// ASCII space, horizontal tab, line feed, vertical tab, form feed, carriage return
    /// and punctuation.
    Separator,
    /// Everything else.
    #[default]
    Unknown,
}

impl From<u8> for Class {
    fn from(byte: u8) -> Self {
        match byte {
            9..=13 => Self::Separator,    // Whitespaces
            32..=47 => Self::Separator,   // Space and punctuations
            48..=57 => Self::Digit,       // Digits
            58..=64 => Self::Separator,   // Punctuations
            65..=90 => Self::Uppercase,   // Uppercases
            91..=96 => Self::Separator,   // Punctuations
            97..=122 => Self::Lowercase,  // Lowercases
            123..=126 => Self::Separator, // Punctuations
            _ => Self::Unknown,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Byte                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Accepted {
    pub index: usize,
    pub needle_start_bonus: bool,
    pub haystack_start_bonus: bool,
    pub uppercase_bonus: bool,
    pub space_as_separator_malus: bool,
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Accept {
    pub byte: u8,
    pub uppercase_bonus: bool,
    pub space_as_separator_malus: bool,
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct NeedleByte {
    pub byte: u8,
    pub is_start: bool,
    pub is_separator: bool,
    pub accepted: Accepted,
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct HaystackByte {
    pub index: usize,
    pub is_start: bool,
    pub accepts: (Accept, Option<Accept>),
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Byte {
    pub byte: u8,
    pub class: Class,
    pub accepts: (Accept, Option<Accept>),
}

impl From<u8> for Byte {
    fn from(byte: u8) -> Self {
        let class = Class::from(byte);

        Self {
            byte,
            class,
            accepts: match class {
                Class::Uppercase => (
                    Accept {
                        byte,
                        uppercase_bonus: true,
                        space_as_separator_malus: false,
                    },
                    Some(Accept {
                        byte: byte + 32, // Lowercase
                        uppercase_bonus: false,
                        space_as_separator_malus: false,
                    }),
                ),
                Class::Lowercase => (
                    Accept {
                        byte,
                        uppercase_bonus: false,
                        space_as_separator_malus: false,
                    },
                    Some(Accept {
                        byte: byte - 32, // Uppercase
                        uppercase_bonus: false,
                        space_as_separator_malus: false,
                    }),
                ),
                Class::Digit => (
                    Accept {
                        byte,
                        uppercase_bonus: false,
                        space_as_separator_malus: false,
                    },
                    None,
                ),
                Class::Separator => (
                    Accept {
                        byte,
                        uppercase_bonus: false,
                        space_as_separator_malus: false,
                    },
                    (byte != b' ').then_some(Accept {
                        byte: b' ',
                        uppercase_bonus: false,
                        space_as_separator_malus: true,
                    }),
                ),
                Class::Unknown => (
                    Accept {
                        byte,
                        uppercase_bonus: false,
                        space_as_separator_malus: false,
                    },
                    None,
                ),
            },
        }
    }
}

impl Byte {
    pub fn parse(
        str: &str,
        mut callback: impl FnMut(/* index */ usize, Self, /* is_start */ bool),
    ) {
        let mut it = str
            .as_bytes()
            .iter()
            .copied()
            .map(Self::from)
            .enumerate()
            .peekable();

        let mut prev_byte = if let Some((_, byte)) = it.next() {
            callback(0, byte, true);
            byte
        } else {
            return;
        };

        while let Some((index, byte)) = it.next() {
            callback(
                index,
                byte,
                match byte.class {
                    Class::Uppercase => match prev_byte.class {
                        Class::Uppercase => {
                            matches!(it.peek(), Some((_, peek)) if peek.class == Class::Lowercase)
                        }
                        _ => true,
                    },
                    Class::Lowercase => {
                        !matches!(prev_byte.class, Class::Uppercase | Class::Lowercase)
                    }
                    Class::Digit => !matches!(prev_byte.class, Class::Digit),
                    Class::Separator => !matches!(prev_byte.class, Class::Separator),
                    Class::Unknown => !matches!(prev_byte.class, Class::Unknown),
                },
            );

            prev_byte = byte;
        }
    }
}
