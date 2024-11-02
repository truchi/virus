pub mod document;
pub mod editor;
pub mod fuzzy;
pub mod rope {
    pub use cursor::*;
    pub use cursors::chunk::*;
    pub use cursors::grapheme::*;
    pub use cursors::word::*;
    pub use edit::*;
    pub use selection::*;
    pub use text::*;

    mod cursor;
    mod cursors {
        pub mod chunk;
        pub mod grapheme;
        pub mod word;
    }
    mod edit;
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
