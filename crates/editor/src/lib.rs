pub mod ast {
    mod highlights;
    mod queries;

    pub use highlights::*;
    pub use queries::*;
}
pub mod document;
pub mod editor;
pub mod fuzzy;
pub mod mode;
pub mod rope {
    mod cursor;
    mod edit;
    mod graphemes;
    mod search;
    mod segmentation;
    mod selection;

    pub use cursor::*;
    pub use edit::*;
    pub use graphemes::*;
    pub use search::*;
    pub use segmentation::*;
    pub use selection::*;
}
pub mod history;

use smol_str::SmolStr;

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[macro_export]
macro_rules! ids {
    (
        $(#[$ids_doc:meta])? $ids_vis:vis $Ids:ident,
        $(#[$id_doc:meta])?  $id_vis:vis  $Id:ident $(,)?
    ) => {
        $(#[$ids_doc])?
        #[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
        $ids_vis struct $Ids(usize);

        impl $Ids {
            $ids_vis fn id(&mut self) -> $Id {
                let id = self.0;
                self.0 += 1;
                $Id(id)
            }
        }

        $(#[$id_doc])?
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
        $id_vis struct $Id(usize);

        impl $Id {
            $id_vis fn id(&self) -> usize {
                self.0
            }
        }
    };
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub fn add_in_range(end: usize, at: usize, add: usize, wrap: bool) -> usize {
    debug_assert!((0..end).contains(&at));

    let result = at + add;

    let result = if wrap {
        result % end
    } else {
        result.clamp(0, end - 1)
    };

    debug_assert!((0..end).contains(&result));
    result
}

pub fn sub_in_range(end: usize, at: usize, sub: usize, wrap: bool) -> usize {
    debug_assert!((0..end).contains(&at));

    let result = if sub <= at {
        at - sub
    } else if wrap {
        end - ((sub - at) % end)
    } else {
        0
    };

    debug_assert!((0..end).contains(&result));
    result
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           StrOrSmol                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum StrOrSmol<'a> {
    Str(&'a str),
    Smol(SmolStr),
}

impl<'a> StrOrSmol<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            StrOrSmol::Str(str) => str,
            StrOrSmol::Smol(smol) => smol.as_str(),
        }
    }
}
