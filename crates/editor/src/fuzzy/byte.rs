use super::Accept;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Class                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

use Class::*;

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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Byte                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Byte {
    pub byte: u8,
    pub class: Class,
    pub accepts: (Accept, Option<Accept>),
}

impl Byte {
    pub const TABLE: [Self; 256] = TABLE;

    pub fn new(byte: u8) -> Self {
        Self::TABLE[byte as usize]
    }

    pub fn parse(
        str: &str,
        mut callback: impl FnMut(/* index */ usize, Self, /* is_start */ bool),
    ) {
        let mut it = str
            .as_bytes()
            .iter()
            .copied()
            .map(|byte| Self::new(byte))
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
                    Uppercase => match prev_byte.class {
                        Uppercase => {
                            matches!(it.peek(), Some((_, peek)) if peek.class == Lowercase)
                        }
                        _ => true,
                    },
                    Lowercase => !matches!(prev_byte.class, Uppercase | Lowercase),
                    Digit => !matches!(prev_byte.class, Digit),
                    Separator => !matches!(prev_byte.class, Separator),
                    Unknown => !matches!(prev_byte.class, Unknown),
                },
            );

            prev_byte = byte;
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Table                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

const TABLE: [Byte; 256] = [
    Byte {
        byte: 0,
        class: Unknown,
        accepts: (
            Accept {
                byte: 0,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 1,
        class: Unknown,
        accepts: (
            Accept {
                byte: 1,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 2,
        class: Unknown,
        accepts: (
            Accept {
                byte: 2,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 3,
        class: Unknown,
        accepts: (
            Accept {
                byte: 3,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 4,
        class: Unknown,
        accepts: (
            Accept {
                byte: 4,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 5,
        class: Unknown,
        accepts: (
            Accept {
                byte: 5,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 6,
        class: Unknown,
        accepts: (
            Accept {
                byte: 6,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 7,
        class: Unknown,
        accepts: (
            Accept {
                byte: 7,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 8,
        class: Unknown,
        accepts: (
            Accept {
                byte: 8,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 9,
        class: Separator,
        accepts: (
            Accept {
                byte: 9,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 10,
        class: Separator,
        accepts: (
            Accept {
                byte: 10,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 11,
        class: Separator,
        accepts: (
            Accept {
                byte: 11,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 12,
        class: Separator,
        accepts: (
            Accept {
                byte: 12,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 13,
        class: Separator,
        accepts: (
            Accept {
                byte: 13,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 14,
        class: Unknown,
        accepts: (
            Accept {
                byte: 14,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 15,
        class: Unknown,
        accepts: (
            Accept {
                byte: 15,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 16,
        class: Unknown,
        accepts: (
            Accept {
                byte: 16,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 17,
        class: Unknown,
        accepts: (
            Accept {
                byte: 17,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 18,
        class: Unknown,
        accepts: (
            Accept {
                byte: 18,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 19,
        class: Unknown,
        accepts: (
            Accept {
                byte: 19,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 20,
        class: Unknown,
        accepts: (
            Accept {
                byte: 20,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 21,
        class: Unknown,
        accepts: (
            Accept {
                byte: 21,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 22,
        class: Unknown,
        accepts: (
            Accept {
                byte: 22,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 23,
        class: Unknown,
        accepts: (
            Accept {
                byte: 23,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 24,
        class: Unknown,
        accepts: (
            Accept {
                byte: 24,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 25,
        class: Unknown,
        accepts: (
            Accept {
                byte: 25,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 26,
        class: Unknown,
        accepts: (
            Accept {
                byte: 26,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 27,
        class: Unknown,
        accepts: (
            Accept {
                byte: 27,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 28,
        class: Unknown,
        accepts: (
            Accept {
                byte: 28,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 29,
        class: Unknown,
        accepts: (
            Accept {
                byte: 29,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 30,
        class: Unknown,
        accepts: (
            Accept {
                byte: 30,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 31,
        class: Unknown,
        accepts: (
            Accept {
                byte: 31,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 32,
        class: Separator,
        accepts: (
            Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 33,
        class: Separator,
        accepts: (
            Accept {
                byte: 33,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 34,
        class: Separator,
        accepts: (
            Accept {
                byte: 34,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 35,
        class: Separator,
        accepts: (
            Accept {
                byte: 35,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 36,
        class: Separator,
        accepts: (
            Accept {
                byte: 36,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 37,
        class: Separator,
        accepts: (
            Accept {
                byte: 37,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 38,
        class: Separator,
        accepts: (
            Accept {
                byte: 38,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 39,
        class: Separator,
        accepts: (
            Accept {
                byte: 39,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 40,
        class: Separator,
        accepts: (
            Accept {
                byte: 40,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 41,
        class: Separator,
        accepts: (
            Accept {
                byte: 41,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 42,
        class: Separator,
        accepts: (
            Accept {
                byte: 42,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 43,
        class: Separator,
        accepts: (
            Accept {
                byte: 43,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 44,
        class: Separator,
        accepts: (
            Accept {
                byte: 44,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 45,
        class: Separator,
        accepts: (
            Accept {
                byte: 45,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 46,
        class: Separator,
        accepts: (
            Accept {
                byte: 46,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 47,
        class: Separator,
        accepts: (
            Accept {
                byte: 47,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 48,
        class: Digit,
        accepts: (
            Accept {
                byte: 48,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 49,
        class: Digit,
        accepts: (
            Accept {
                byte: 49,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 50,
        class: Digit,
        accepts: (
            Accept {
                byte: 50,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 51,
        class: Digit,
        accepts: (
            Accept {
                byte: 51,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 52,
        class: Digit,
        accepts: (
            Accept {
                byte: 52,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 53,
        class: Digit,
        accepts: (
            Accept {
                byte: 53,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 54,
        class: Digit,
        accepts: (
            Accept {
                byte: 54,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 55,
        class: Digit,
        accepts: (
            Accept {
                byte: 55,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 56,
        class: Digit,
        accepts: (
            Accept {
                byte: 56,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 57,
        class: Digit,
        accepts: (
            Accept {
                byte: 57,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 58,
        class: Separator,
        accepts: (
            Accept {
                byte: 58,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 59,
        class: Separator,
        accepts: (
            Accept {
                byte: 59,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 60,
        class: Separator,
        accepts: (
            Accept {
                byte: 60,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 61,
        class: Separator,
        accepts: (
            Accept {
                byte: 61,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 62,
        class: Separator,
        accepts: (
            Accept {
                byte: 62,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 63,
        class: Separator,
        accepts: (
            Accept {
                byte: 63,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 64,
        class: Separator,
        accepts: (
            Accept {
                byte: 64,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 65,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 65,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 97,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 66,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 66,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 98,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 67,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 67,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 99,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 68,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 68,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 100,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 69,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 69,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 101,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 70,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 70,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 102,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 71,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 71,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 103,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 72,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 72,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 104,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 73,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 73,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 105,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 74,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 74,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 106,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 75,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 75,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 107,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 76,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 76,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 108,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 77,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 77,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 109,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 78,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 78,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 110,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 79,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 79,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 111,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 80,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 80,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 112,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 81,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 81,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 113,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 82,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 82,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 114,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 83,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 83,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 115,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 84,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 84,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 116,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 85,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 85,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 117,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 86,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 86,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 118,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 87,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 87,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 119,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 88,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 88,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 120,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 89,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 89,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 121,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 90,
        class: Uppercase,
        accepts: (
            Accept {
                byte: 90,
                uppercase_bonus: true,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 122,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 91,
        class: Separator,
        accepts: (
            Accept {
                byte: 91,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 92,
        class: Separator,
        accepts: (
            Accept {
                byte: 92,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 93,
        class: Separator,
        accepts: (
            Accept {
                byte: 93,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 94,
        class: Separator,
        accepts: (
            Accept {
                byte: 94,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 95,
        class: Separator,
        accepts: (
            Accept {
                byte: 95,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 96,
        class: Separator,
        accepts: (
            Accept {
                byte: 96,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 97,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 97,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 65,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 98,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 98,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 66,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 99,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 99,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 67,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 100,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 100,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 68,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 101,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 101,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 69,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 102,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 102,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 70,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 103,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 103,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 71,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 104,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 104,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 72,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 105,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 105,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 73,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 106,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 106,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 74,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 107,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 107,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 75,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 108,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 108,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 76,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 109,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 109,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 77,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 110,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 110,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 78,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 111,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 111,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 79,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 112,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 112,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 80,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 113,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 113,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 81,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 114,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 114,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 82,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 115,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 115,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 83,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 116,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 116,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 84,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 117,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 117,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 85,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 118,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 118,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 86,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 119,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 119,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 87,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 120,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 120,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 88,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 121,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 121,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 89,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 122,
        class: Lowercase,
        accepts: (
            Accept {
                byte: 122,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 90,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            }),
        ),
    },
    Byte {
        byte: 123,
        class: Separator,
        accepts: (
            Accept {
                byte: 123,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 124,
        class: Separator,
        accepts: (
            Accept {
                byte: 124,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 125,
        class: Separator,
        accepts: (
            Accept {
                byte: 125,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 126,
        class: Separator,
        accepts: (
            Accept {
                byte: 126,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            Some(Accept {
                byte: 32,
                uppercase_bonus: false,
                space_as_separator_malus: true,
            }),
        ),
    },
    Byte {
        byte: 127,
        class: Unknown,
        accepts: (
            Accept {
                byte: 127,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 128,
        class: Unknown,
        accepts: (
            Accept {
                byte: 128,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 129,
        class: Unknown,
        accepts: (
            Accept {
                byte: 129,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 130,
        class: Unknown,
        accepts: (
            Accept {
                byte: 130,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 131,
        class: Unknown,
        accepts: (
            Accept {
                byte: 131,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 132,
        class: Unknown,
        accepts: (
            Accept {
                byte: 132,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 133,
        class: Unknown,
        accepts: (
            Accept {
                byte: 133,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 134,
        class: Unknown,
        accepts: (
            Accept {
                byte: 134,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 135,
        class: Unknown,
        accepts: (
            Accept {
                byte: 135,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 136,
        class: Unknown,
        accepts: (
            Accept {
                byte: 136,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 137,
        class: Unknown,
        accepts: (
            Accept {
                byte: 137,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 138,
        class: Unknown,
        accepts: (
            Accept {
                byte: 138,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 139,
        class: Unknown,
        accepts: (
            Accept {
                byte: 139,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 140,
        class: Unknown,
        accepts: (
            Accept {
                byte: 140,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 141,
        class: Unknown,
        accepts: (
            Accept {
                byte: 141,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 142,
        class: Unknown,
        accepts: (
            Accept {
                byte: 142,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 143,
        class: Unknown,
        accepts: (
            Accept {
                byte: 143,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 144,
        class: Unknown,
        accepts: (
            Accept {
                byte: 144,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 145,
        class: Unknown,
        accepts: (
            Accept {
                byte: 145,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 146,
        class: Unknown,
        accepts: (
            Accept {
                byte: 146,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 147,
        class: Unknown,
        accepts: (
            Accept {
                byte: 147,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 148,
        class: Unknown,
        accepts: (
            Accept {
                byte: 148,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 149,
        class: Unknown,
        accepts: (
            Accept {
                byte: 149,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 150,
        class: Unknown,
        accepts: (
            Accept {
                byte: 150,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 151,
        class: Unknown,
        accepts: (
            Accept {
                byte: 151,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 152,
        class: Unknown,
        accepts: (
            Accept {
                byte: 152,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 153,
        class: Unknown,
        accepts: (
            Accept {
                byte: 153,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 154,
        class: Unknown,
        accepts: (
            Accept {
                byte: 154,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 155,
        class: Unknown,
        accepts: (
            Accept {
                byte: 155,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 156,
        class: Unknown,
        accepts: (
            Accept {
                byte: 156,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 157,
        class: Unknown,
        accepts: (
            Accept {
                byte: 157,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 158,
        class: Unknown,
        accepts: (
            Accept {
                byte: 158,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 159,
        class: Unknown,
        accepts: (
            Accept {
                byte: 159,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 160,
        class: Unknown,
        accepts: (
            Accept {
                byte: 160,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 161,
        class: Unknown,
        accepts: (
            Accept {
                byte: 161,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 162,
        class: Unknown,
        accepts: (
            Accept {
                byte: 162,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 163,
        class: Unknown,
        accepts: (
            Accept {
                byte: 163,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 164,
        class: Unknown,
        accepts: (
            Accept {
                byte: 164,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 165,
        class: Unknown,
        accepts: (
            Accept {
                byte: 165,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 166,
        class: Unknown,
        accepts: (
            Accept {
                byte: 166,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 167,
        class: Unknown,
        accepts: (
            Accept {
                byte: 167,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 168,
        class: Unknown,
        accepts: (
            Accept {
                byte: 168,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 169,
        class: Unknown,
        accepts: (
            Accept {
                byte: 169,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 170,
        class: Unknown,
        accepts: (
            Accept {
                byte: 170,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 171,
        class: Unknown,
        accepts: (
            Accept {
                byte: 171,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 172,
        class: Unknown,
        accepts: (
            Accept {
                byte: 172,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 173,
        class: Unknown,
        accepts: (
            Accept {
                byte: 173,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 174,
        class: Unknown,
        accepts: (
            Accept {
                byte: 174,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 175,
        class: Unknown,
        accepts: (
            Accept {
                byte: 175,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 176,
        class: Unknown,
        accepts: (
            Accept {
                byte: 176,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 177,
        class: Unknown,
        accepts: (
            Accept {
                byte: 177,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 178,
        class: Unknown,
        accepts: (
            Accept {
                byte: 178,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 179,
        class: Unknown,
        accepts: (
            Accept {
                byte: 179,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 180,
        class: Unknown,
        accepts: (
            Accept {
                byte: 180,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 181,
        class: Unknown,
        accepts: (
            Accept {
                byte: 181,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 182,
        class: Unknown,
        accepts: (
            Accept {
                byte: 182,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 183,
        class: Unknown,
        accepts: (
            Accept {
                byte: 183,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 184,
        class: Unknown,
        accepts: (
            Accept {
                byte: 184,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 185,
        class: Unknown,
        accepts: (
            Accept {
                byte: 185,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 186,
        class: Unknown,
        accepts: (
            Accept {
                byte: 186,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 187,
        class: Unknown,
        accepts: (
            Accept {
                byte: 187,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 188,
        class: Unknown,
        accepts: (
            Accept {
                byte: 188,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 189,
        class: Unknown,
        accepts: (
            Accept {
                byte: 189,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 190,
        class: Unknown,
        accepts: (
            Accept {
                byte: 190,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 191,
        class: Unknown,
        accepts: (
            Accept {
                byte: 191,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 192,
        class: Unknown,
        accepts: (
            Accept {
                byte: 192,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 193,
        class: Unknown,
        accepts: (
            Accept {
                byte: 193,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 194,
        class: Unknown,
        accepts: (
            Accept {
                byte: 194,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 195,
        class: Unknown,
        accepts: (
            Accept {
                byte: 195,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 196,
        class: Unknown,
        accepts: (
            Accept {
                byte: 196,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 197,
        class: Unknown,
        accepts: (
            Accept {
                byte: 197,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 198,
        class: Unknown,
        accepts: (
            Accept {
                byte: 198,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 199,
        class: Unknown,
        accepts: (
            Accept {
                byte: 199,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 200,
        class: Unknown,
        accepts: (
            Accept {
                byte: 200,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 201,
        class: Unknown,
        accepts: (
            Accept {
                byte: 201,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 202,
        class: Unknown,
        accepts: (
            Accept {
                byte: 202,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 203,
        class: Unknown,
        accepts: (
            Accept {
                byte: 203,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 204,
        class: Unknown,
        accepts: (
            Accept {
                byte: 204,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 205,
        class: Unknown,
        accepts: (
            Accept {
                byte: 205,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 206,
        class: Unknown,
        accepts: (
            Accept {
                byte: 206,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 207,
        class: Unknown,
        accepts: (
            Accept {
                byte: 207,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 208,
        class: Unknown,
        accepts: (
            Accept {
                byte: 208,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 209,
        class: Unknown,
        accepts: (
            Accept {
                byte: 209,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 210,
        class: Unknown,
        accepts: (
            Accept {
                byte: 210,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 211,
        class: Unknown,
        accepts: (
            Accept {
                byte: 211,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 212,
        class: Unknown,
        accepts: (
            Accept {
                byte: 212,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 213,
        class: Unknown,
        accepts: (
            Accept {
                byte: 213,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 214,
        class: Unknown,
        accepts: (
            Accept {
                byte: 214,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 215,
        class: Unknown,
        accepts: (
            Accept {
                byte: 215,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 216,
        class: Unknown,
        accepts: (
            Accept {
                byte: 216,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 217,
        class: Unknown,
        accepts: (
            Accept {
                byte: 217,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 218,
        class: Unknown,
        accepts: (
            Accept {
                byte: 218,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 219,
        class: Unknown,
        accepts: (
            Accept {
                byte: 219,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 220,
        class: Unknown,
        accepts: (
            Accept {
                byte: 220,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 221,
        class: Unknown,
        accepts: (
            Accept {
                byte: 221,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 222,
        class: Unknown,
        accepts: (
            Accept {
                byte: 222,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 223,
        class: Unknown,
        accepts: (
            Accept {
                byte: 223,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 224,
        class: Unknown,
        accepts: (
            Accept {
                byte: 224,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 225,
        class: Unknown,
        accepts: (
            Accept {
                byte: 225,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 226,
        class: Unknown,
        accepts: (
            Accept {
                byte: 226,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 227,
        class: Unknown,
        accepts: (
            Accept {
                byte: 227,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 228,
        class: Unknown,
        accepts: (
            Accept {
                byte: 228,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 229,
        class: Unknown,
        accepts: (
            Accept {
                byte: 229,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 230,
        class: Unknown,
        accepts: (
            Accept {
                byte: 230,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 231,
        class: Unknown,
        accepts: (
            Accept {
                byte: 231,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 232,
        class: Unknown,
        accepts: (
            Accept {
                byte: 232,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 233,
        class: Unknown,
        accepts: (
            Accept {
                byte: 233,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 234,
        class: Unknown,
        accepts: (
            Accept {
                byte: 234,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 235,
        class: Unknown,
        accepts: (
            Accept {
                byte: 235,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 236,
        class: Unknown,
        accepts: (
            Accept {
                byte: 236,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 237,
        class: Unknown,
        accepts: (
            Accept {
                byte: 237,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 238,
        class: Unknown,
        accepts: (
            Accept {
                byte: 238,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 239,
        class: Unknown,
        accepts: (
            Accept {
                byte: 239,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 240,
        class: Unknown,
        accepts: (
            Accept {
                byte: 240,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 241,
        class: Unknown,
        accepts: (
            Accept {
                byte: 241,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 242,
        class: Unknown,
        accepts: (
            Accept {
                byte: 242,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 243,
        class: Unknown,
        accepts: (
            Accept {
                byte: 243,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 244,
        class: Unknown,
        accepts: (
            Accept {
                byte: 244,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 245,
        class: Unknown,
        accepts: (
            Accept {
                byte: 245,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 246,
        class: Unknown,
        accepts: (
            Accept {
                byte: 246,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 247,
        class: Unknown,
        accepts: (
            Accept {
                byte: 247,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 248,
        class: Unknown,
        accepts: (
            Accept {
                byte: 248,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 249,
        class: Unknown,
        accepts: (
            Accept {
                byte: 249,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 250,
        class: Unknown,
        accepts: (
            Accept {
                byte: 250,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 251,
        class: Unknown,
        accepts: (
            Accept {
                byte: 251,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 252,
        class: Unknown,
        accepts: (
            Accept {
                byte: 252,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 253,
        class: Unknown,
        accepts: (
            Accept {
                byte: 253,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 254,
        class: Unknown,
        accepts: (
            Accept {
                byte: 254,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
    Byte {
        byte: 255,
        class: Unknown,
        accepts: (
            Accept {
                byte: 255,
                uppercase_bonus: false,
                space_as_separator_malus: false,
            },
            None,
        ),
    },
];

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;

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

    impl From<u8> for Byte {
        fn from(byte: u8) -> Self {
            let class = Class::from(byte);

            Self {
                byte,
                class,
                accepts: match class {
                    Uppercase => (
                        Accept {
                            byte,
                            uppercase_bonus: true,
                            space_as_separator_malus: false,
                        },
                        Some(Accept {
                            byte: Self::lower(byte),
                            uppercase_bonus: false,
                            space_as_separator_malus: false,
                        }),
                    ),
                    Lowercase => (
                        Accept {
                            byte,
                            uppercase_bonus: false,
                            space_as_separator_malus: false,
                        },
                        Some(Accept {
                            byte: Self::upper(byte),
                            uppercase_bonus: false,
                            space_as_separator_malus: false,
                        }),
                    ),
                    Digit => (
                        Accept {
                            byte,
                            uppercase_bonus: false,
                            space_as_separator_malus: false,
                        },
                        None,
                    ),
                    Separator => (
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
                    Unknown => (
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
        fn table() -> [Byte; 256] {
            let mut table = [Byte::default(); 256];

            for byte in 0..=u8::MAX {
                table[byte as usize] = Byte::from(byte);
            }

            table
        }

        fn lower(byte: u8) -> u8 {
            debug_assert!(Class::from(byte) == Uppercase);
            byte + 32
        }

        fn upper(byte: u8) -> u8 {
            debug_assert!(Class::from(byte) == Lowercase);
            byte - 32
        }
    }

    #[test]
    fn table() {
        assert!(Byte::TABLE == Byte::table());
    }
}
