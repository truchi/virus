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
