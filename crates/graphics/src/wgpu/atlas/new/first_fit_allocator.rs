use super::*;
use std::{collections::HashMap, hash::Hash};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Item                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A [`FirstFitAllocator`] item.
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

/// A [`FirstFitAllocator`] shelf.
#[derive(Default, Debug)]
struct Shelf {
    /// The cross-axis offset.
    offset: u32,
    /// The occupied main-axis size.
    main: u32,
    /// The cross-axis size of the largest item.
    cross: u32,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       FirstFitAllocator                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A first-fit atlas allocator (non bucketed, without deallocation).
pub struct FirstFitAllocator<A: Axis, K: Eq + Hash, V> {
    /// The width of the atlas.
    width: u32,
    /// The height of the atlas.
    height: u32,
    /// The items in the atlas.
    items: HashMap<K, Item<V>>,
    /// The shelves of the atlas.
    shelves: Vec<Shelf>,
    /// Shelf axis.
    _axis: PhantomData<A>,
}

impl<A: Axis, K: Eq + Hash, V> Allocator<K, V, ()> for FirstFitAllocator<A, K, V> {
    fn new(width: u32, height: u32, _: ()) -> Self {
        Self {
            width,
            height,
            items: Default::default(),
            shelves: Default::default(),
            _axis: PhantomData,
        }
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn row(&self) -> u32 {
        self.height
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn get(&self, key: &K) -> Option<([u32; 2], &V)> {
        self.items
            .get(key)
            .map(|item| (A::flip(item.main, item.cross), &item.value))
    }

    fn next_frame(&mut self) {}

    fn insert(&mut self, key: K, value: V, [width, height]: [u32; 2]) -> Result<Option<V>, (K, V)> {
        // Check dimensions
        if width > self.width || height > self.height {
            return Err((key, value));
        }

        // Lookup cache
        if self.items.get(&key).is_some() {
            return Ok(Some(value));
        }

        // Insert (or fail)
        self.try_insert(key, value, [width, height])?;
        Ok(None)
    }

    fn clear(&mut self) {
        self.items.clear();
        self.shelves.clear();
    }

    fn clear_and_resize(&mut self, width: u32, height: u32, _: ()) {
        self.clear();

        self.width = width;
        self.height = height;
    }
}

/// Private.
impl<A: Axis, K: Eq + Hash, V> FirstFitAllocator<A, K, V> {
    /// Tries to insert an item.
    fn try_insert(&mut self, key: K, value: V, [width, height]: [u32; 2]) -> Result<(), (K, V)> {
        // NOTE: this MUST be checked before calling this function
        debug_assert!(width <= self.width);
        debug_assert!(height <= self.height);

        let [self_main, self_cross] = A::flip(self.width, self.height);
        let [main, cross] = A::flip(width, height);

        for shelf in self.shelves.iter_mut().rev().skip(1).rev() {
            // Fits in closed shelf
            if (cross <= shelf.cross) && (main <= self_main - shelf.main) {
                self.items.insert(
                    key,
                    Item {
                        main: shelf.main,
                        cross: shelf.offset,
                        value,
                    },
                );
                shelf.main += main;

                return Ok(());
            }
        }

        let offset = if let Some(shelf) = self.shelves.last_mut() {
            // Fits in open shelf
            if (cross <= self_cross - shelf.offset) && (main <= self_main - shelf.main) {
                self.items.insert(
                    key,
                    Item {
                        main: shelf.main,
                        cross: shelf.offset,
                        value,
                    },
                );
                shelf.main += main;
                shelf.cross = shelf.cross.max(cross);

                return Ok(());
            }

            shelf.offset + shelf.cross
        } else {
            0
        };

        // Fits in new shelf
        if cross <= self_cross - offset {
            self.items.insert(
                key,
                Item {
                    main: 0,
                    cross: offset,
                    value,
                },
            );
            self.shelves.push(Shelf {
                offset,
                main,
                cross,
            });

            return Ok(());
        }

        Err((key, value))
    }
}
