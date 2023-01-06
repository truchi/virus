#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Eol {
    Cr,
    Lf,
    Crlf,
}

impl Eol {
    /// Returns the underlying `&str`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cr => "\r",
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
        }
    }

    pub fn split_before(str: &str, end: usize) -> (&str, Option<Self>, &str) {
        let (before, after) = str.split_at(end.min(str.len()));

        let bytes = before.as_bytes();
        for (i, byte) in bytes.iter().enumerate() {
            match byte {
                b'\r' => match bytes.get(i + 1) {
                    Some(b'\n') => return (&str[..i], Some(Self::Crlf), &str[i + 2..]),
                    _ => return (&str[..i], Some(Self::Cr), &str[i + 1..]),
                },
                b'\n' => return (&str[..i], Some(Self::Lf), &str[i + 1..]),
                _ => {}
            }
        }

        (before, None, after)
    }

    pub fn leading(str: &str) -> Option<(Self, &str)> {
        let bytes = str.as_bytes();

        match bytes.get(0) {
            Some(b'\r') => match bytes.get(1) {
                Some(b'\n') => Some((Self::Crlf, &str[2..])),
                _ => Some((Self::Cr, &str[1..])),
            },
            Some(b'\n') => Some((Self::Lf, &str[1..])),
            _ => None,
        }
    }

    pub fn option_to_u8(eol: Option<Eol>) -> u8 {
        match eol {
            None => 0,
            Some(Self::Cr) => 1,
            Some(Self::Lf) => 2,
            Some(Self::Crlf) => 3,
        }
    }

    pub fn u8_to_option(u8: u8) -> Option<Self> {
        match u8 {
            0 => None,
            1 => Some(Self::Cr),
            2 => Some(Self::Lf),
            u8 => {
                debug_assert!(u8 == 3);
                Some(Self::Crlf)
            }
        }
    }
}
