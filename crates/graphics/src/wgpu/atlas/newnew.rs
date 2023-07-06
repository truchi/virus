#![allow(unused)] // TODO REMOVE

use std::{collections::HashMap, hash::Hash, marker::PhantomData};
use wgpu::{Texture, TextureDimension, TextureFormat};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Axes                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Shelves axis.
pub trait Axis {
    /// Flips `a` and `b` when [`Vertical`].
    fn flip<T>(a: T, b: T) -> [T; 2];
}

/// Horizontal shelves.
pub enum Horizontal {}

impl Axis for Horizontal {
    fn flip<T>(a: T, b: T) -> [T; 2] {
        [a, b]
    }
}

/// Vertical shelves.
pub enum Vertical {}

impl Axis for Vertical {
    fn flip<T>(a: T, b: T) -> [T; 2] {
        [b, a]
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Item                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An [`Allocator`] item.
#[derive(Copy, Clone, Debug)]
struct Item<V> {
    /// The main-axis coordinate.
    main: u32,
    /// The cross-axis coordinate.
    cross: u32,
    /// The value.
    value: V,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Shelf                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An [`Allocator`] shelf.
#[derive(Copy, Clone, Debug)]
struct Shelf {
    /// The occupied main-axis size.
    main: u32,
    /// The cross-axis size of the largest item.
    cross: u32,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Allocator                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Error type for [`Allocator::insert()`].
pub enum InsertError {
    /// The item is too big for the atlas' width/height/bin dimensions. Resize the atlas.
    WontFit,
    /// The item is too big for the atlas' remaining space. Clear the atlas.
    OutOfSpace,
}

/// An atlas allocator.
#[derive(Clone, Debug)]
pub struct Allocator<A: Axis, K: Clone + Eq + Hash, V> {
    /// The width of the atlas.
    width: u32,
    /// The height of the atlas.
    height: u32,
    /// The bin main-axis size.
    bin: u32,
    /// The bins of the atlas.
    bins: Vec<Vec<Shelf>>,
    /// The items in the atlas.
    items: HashMap<K, Item<V>>,
    /// The shelves' axis.
    _axis: PhantomData<A>,
}

impl<A: Axis, K: Clone + Eq + Hash, V> Allocator<A, K, V> {
    /// Creates a new empty allocator with `width` and `height`, and `bin` main-axis size.
    pub fn new(width: u32, height: u32, bin: u32) -> Self {
        Self {
            width,
            height,
            bin: bin.min(A::flip(width, height)[0]),
            bins: Default::default(),
            items: Default::default(),
            _axis: PhantomData,
        }
    }

    /// Returns the width of the allocator.
    fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the allocator.
    fn height(&self) -> u32 {
        self.height
    }

    /// Returns the bin main-axis size.
    fn bin(&self) -> u32 {
        self.bin
    }

    /// Returns the number of items in the allocator.
    fn len(&self) -> usize {
        self.items.len()
    }

    /// Inserts an item for `key` with `([width, height], value)` provided through `f`
    /// (only called when the item does not exist already).
    ///
    /// If allocation fails, call [`Self::clear()`] before the next frame, or try a larger
    /// allocator.
    fn insert<F: FnOnce() -> ([u32; 2], V)>(
        &mut self,
        key: K,
        f: F,
    ) -> Result<([u32; 2], &V), InsertError> {
        // Insert when not in cache already
        if self.items.get(&key).is_none() {
            let ([width, height], value) = f();
            let [main, cross] = A::flip(width, height);
            let [_, self_cross] = A::flip(self.width, self.height);

            // Check dimensions
            if !(main <= self.bin && cross <= self_cross) {
                return Err(InsertError::WontFit);
            }

            // Insert or fail
            if self.try_insert(&key, value, [main, cross]).is_err() {
                return Err(InsertError::OutOfSpace);
            }
        }

        // Lookup cache
        let item = self.items.get(&key).unwrap();
        Ok((A::flip(item.main, item.cross), &item.value))
    }

    /// Clears the allocator.
    pub fn clear(&mut self) {
        self.items.clear();
        self.bins.clear();
    }

    /// Clears and resizes the allocator.
    pub fn clear_and_resize(&mut self, width: u32, height: u32, bin: u32) {
        self.clear();

        self.width = width;
        self.height = height;
        self.bin = bin.min(A::flip(width, height)[0]);
    }
}

/// Private.
impl<A: Axis, K: Clone + Eq + Hash, V> Allocator<A, K, V> {
    /// Tries to insert an item.
    fn try_insert(&mut self, key: &K, value: V, [main, cross]: [u32; 2]) -> Result<(), ()> {
        let [self_main, self_cross] = A::flip(self.width, self.height);

        // Main-axis position in the atlas
        let mut bin_position = 0;

        for bin in &mut self.bins {
            // Main-axis size
            let bin_size = self.bin.min(self_main - bin_position);

            // Cross-axis position in the bin
            let mut shelf_position = 0;

            if let Some((open, closeds)) = bin.split_last_mut() {
                // Fits in closed shelf?
                for closed in closeds {
                    if (main <= bin_size - closed.main) && (cross <= closed.cross) {
                        self.items.insert(
                            key.clone(),
                            Item {
                                main: bin_position + closed.main,
                                cross: shelf_position,
                                value,
                            },
                        );
                        closed.main += main;

                        return Ok(());
                    }

                    shelf_position += closed.cross;
                }

                // Fits in open shelf?
                if (main <= bin_size - open.main) && (cross <= self_cross - shelf_position) {
                    self.items.insert(
                        key.clone(),
                        Item {
                            main: bin_position + open.main,
                            cross: shelf_position,
                            value,
                        },
                    );
                    open.main += main;
                    open.cross = open.cross.max(cross);

                    return Ok(());
                }

                shelf_position += open.cross;
            }

            // Fits in new shelf?
            if (main <= bin_size) && (cross <= self_cross - shelf_position) {
                self.items.insert(
                    key.clone(),
                    Item {
                        main: bin_position,
                        cross: shelf_position,
                        value,
                    },
                );
                bin.push(Shelf { main, cross });

                return Ok(());
            }

            bin_position += bin_size;
        }

        // Fits in new bin?
        debug_assert!(cross <= self_cross);
        if main <= self.bin.min(self_main - bin_position) {
            self.items.insert(
                key.clone(),
                Item {
                    main: bin_position,
                    cross: 0,
                    value,
                },
            );
            self.bins.push(vec![Shelf { main, cross }]);

            return Ok(());
        }

        Err(())
    }
}
