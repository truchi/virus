use grappe::page::{meta::PageMeta, Page};
use std::mem::size_of;

fn main() {
    dbg!(size_of::<Page>());
    dbg!(Page::LEN);
    dbg!(size_of::<PageMeta>());
}
