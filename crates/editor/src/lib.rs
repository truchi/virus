pub mod document;
pub mod editor;
pub mod fuzzy;
pub mod rope {
    pub use cursor::*;
    pub use cursors::chunk::*;
    pub use cursors::grapheme::*;
    pub use cursors::word::*;
    pub use edit::*;
    pub use graphemes::*;
    pub use segmentation::*;
    pub use selection::*;
    pub use text::*;

    mod cursor;
    mod cursors {
        pub mod chunk;
        pub mod grapheme;
        pub mod word;
    }
    mod edit;
    mod graphemes;
    mod segmentation;
    mod selection;
    mod text;
}
pub mod history;

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
