#![allow(unused)]
// mask/color: Bucketed next fit (good at dealloc, ok at packing)
// blur: Shelf first fit (no dealloc, great at packing)

// mod allocator;
// mod first_fit_allocator;
// mod next_fit_allocator;

// use allocator::Allocator;
// use std::marker::PhantomData;

use std::{
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Axes                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Shelves axis.
pub trait Axis {
    /// Flips `a` and `b` when [`Vertical`].
    fn flip<T>(a: T, b: T) -> [T; 2];

    /// Returns the main-axis value.
    fn main<T>(x: T, y: T) -> T {
        let [main, _] = Self::flip(x, y);
        main
    }

    /// Returns the cross-axis value.
    fn cross<T>(x: T, y: T) -> T {
        let [_, cross] = Self::flip(x, y);
        cross
    }
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
    /// The bucket index.
    bucket: u32,
    /// The shelf index.
    shelf: u32,
    /// The main-axis coordinate.
    main: u32,
    /// The cross-axis coordinate.
    cross: u32,
    /// The value.
    value: V,
}

impl<V> Item<V> {
    fn to_tuple<A: Axis>(&self) -> ([u32; 2], &V) {
        (A::flip(self.main, self.cross), &self.value)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Shelf                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An [`Allocator`] shelf.
#[derive(Copy, Clone, Debug)]
struct Shelf {
    /// The most recent frame that inserted in this shelf.
    frame: u32,
    /// The cross-axis position of the shelf in the bucket.
    position: u32,
    /// The occupied main-axis size.
    main: u32,
    /// The cross-axis size of the largest item.
    cross: u32,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Bucket                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An [`Allocator`] bucket.
#[derive(Clone, Default, Debug)]
struct Bucket {
    /// The most recent frame that inserted in this bucket.
    frame: u32,
    /// The count of items that are in this bucket for this `frame`.
    count: u32,
    /// The main-axis position of the bucket in the atlas.
    position: u32,
    /// The main-axis size of the bucket.
    size: u32,
    /// The shelves of the bucket.
    shelves: Vec<Shelf>,
    /// The older bucket index.
    prev: Option<u32>,
    /// The newer bucket index.
    next: Option<u32>,
}

impl Index<u32> for Bucket {
    type Output = Shelf;

    fn index(&self, index: u32) -> &Self::Output {
        &self.shelves[usize::try_from(index).unwrap()]
    }
}

impl IndexMut<u32> for Bucket {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.shelves[usize::try_from(index).unwrap()]
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Buckets                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// The [`Allocator`] buckets.
#[derive(Clone, Debug)]
struct Buckets {
    /// The buckets.
    buckets: Vec<Bucket>,
    /// The oldest bucket index.
    first: u32,
    /// The newest bucket index.
    last: u32,
}

impl Buckets {
    fn new(bucket: u32, total: u32) -> Self {
        let mut buckets = Buckets {
            buckets: Vec::new(),
            first: 0,
            last: 0,
        };
        buckets.clear_and_resize(bucket, total);
        buckets
    }

    fn oldest_to_newest(&self) -> impl '_ + Iterator<Item = u32> {
        let mut first = None;
        let mut last = None;

        std::iter::from_fn(move || match (first, last) {
            (None, None) => None,
            (Some(f), Some(l)) => {
                if f == l {
                    (first, last) = (None, None);
                } else {
                    first = self[f].next;
                }

                Some(f)
            }
            _ => unreachable!(),
        })
    }

    fn newest_to_oldest(&self) -> impl '_ + Iterator<Item = u32> {
        let mut first = None;
        let mut last = None;

        std::iter::from_fn(move || match (first, last) {
            (None, None) => None,
            (Some(f), Some(l)) => {
                if l == f {
                    (first, last) = (None, None);
                } else {
                    last = self[l].prev;
                }

                Some(l)
            }
            _ => unreachable!(),
        })
    }

    fn oldest_mut(&mut self) -> &mut Bucket {
        let first = self.first;
        &mut self[first]
    }

    fn touch(&mut self, index: u32, frame: u32) {
        debug_assert!(self[self.last].frame <= frame);

        let bucket = &mut self[index];
        let (prev, next) = (bucket.prev, bucket.next);
        let count = if bucket.frame == frame {
            bucket.count + 1
        } else {
            1
        };

        // Touch
        bucket.frame = frame;
        bucket.count = count;

        // Detach
        bucket.prev = None;
        bucket.next = None;

        match (prev, next) {
            // Only bucket
            (None, None) => return,
            // First bucket
            (None, Some(next)) => {
                self.first = next;
                self[next].prev = None;
            }
            // Last bucket
            (Some(prev), None) => {
                self.last = prev;
                self[prev].next = None;
            }
            // Middle bucket
            (Some(prev), Some(next)) => {
                self[prev].next = Some(next);
                self[next].prev = Some(prev);
            }
        }

        // Find insertion point
        let prev = self
            .oldest_to_newest()
            .map(|index| (index, &self[index]))
            .skip_while(|(_, bucket)| (bucket.frame, bucket.count) > (frame, count))
            .next();

        // Re-attach
        if let Some((prev, bucket)) = prev {
            match bucket.next {
                // Middle bucket
                Some(next) => {
                    self[index].prev = Some(prev);
                    self[index].next = Some(next);
                    self[prev].next = Some(index);
                    self[next].prev = Some(index);
                }
                // Last bucket
                None => {
                    self.last = index;
                    self[index].prev = Some(prev);
                    self[prev].next = Some(index);
                }
            }
        } else {
            // First bucket
            let first = self.first;
            self.first = index;
            self[index].next = Some(first);
            self[first].prev = Some(index);
        }
    }

    fn clear_and_resize(&mut self, bucket: u32, total: u32) {
        debug_assert!(bucket != 0 && bucket <= total);

        let (len, trunc) = {
            let len = total / bucket;
            let trunc = total - len * bucket;

            if trunc == 0 {
                (len, bucket)
            } else {
                (len + 1, trunc)
            }
        };

        self.buckets.clear();
        self.buckets.extend((0..len).map(|i| Bucket {
            frame: 0,
            count: 0,
            position: i * bucket,
            size: bucket,
            shelves: Vec::new(),
            prev: i.checked_sub(1),
            next: Some(i + 1),
        }));

        let last = self.buckets.last_mut().unwrap();
        last.size = trunc;
        last.next = None;

        self.first = 0;
        self.last = len - 1;
    }
}

impl Index<u32> for Buckets {
    type Output = Bucket;

    fn index(&self, index: u32) -> &Self::Output {
        &self.buckets[usize::try_from(index).unwrap()]
    }
}

impl IndexMut<u32> for Buckets {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.buckets[usize::try_from(index).unwrap()]
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           Allocator                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An atlas allocator.
#[derive(Clone, Debug)]
pub struct Allocator<A: Axis, K: Eq + Hash, V> {
    /// The bucket main-axis size.
    bucket: u32,
    /// The width of the atlas.
    width: u32,
    /// The height of the atlas.
    height: u32,
    /// The items in the atlas.
    items: HashMap<K, Item<V>>,
    /// The buckets of the atlas.
    buckets: Buckets,
    /// The current frame.
    frame: u32,
    /// The shelves' axis.
    _axis: PhantomData<A>,
}

impl<A: Axis, K: Eq + Hash, V> Allocator<A, K, V> {
    /// Creates a new empty allocator with `width` and `height`, and `bucket` cross-axis size.
    pub fn new(width: u32, height: u32, bucket: u32) -> Self {
        assert!(width != 0 && height != 0 && bucket != 0);

        let main = A::main(width, height);
        let bucket = bucket.min(main);

        Self {
            bucket,
            width,
            height,
            items: HashMap::new(),
            buckets: Buckets::new(bucket, main),
            frame: 0,
            _axis: PhantomData,
        }
    }

    /// Returns the width of the atlas.
    fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the atlas.
    fn height(&self) -> u32 {
        self.height
    }

    /// Returns the bucket main-axis size.
    fn bucket(&self) -> u32 {
        self.bucket
    }

    /// Returns the number of items in the atlas.
    fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns the `[x, y]` position and value of the item for `key`.
    ///
    /// If this item is to be used in the current frame, you ***MUST*** call [`Self::insert()`].
    fn get(&self, key: &K) -> Option<([u32; 2], &V)> {
        self.items.get(key).map(Item::to_tuple::<A>)
    }

    /// Marks the beginning of a new frame.
    ///
    /// You ***MUST*** call this function inbewteen frames, and ***ONLY*** inbewteen frames,
    /// ***UNLESS*** you clear inbetween frames.
    ///
    /// Clears the allocator when the underlying `u32` would overflow (2.2 years at 60pfs).
    fn next_frame(&mut self) {
        if let Some(frame) = self.frame.checked_add(1) {
            self.frame = frame;
        } else {
            self.clear();
        }
    }

    /// Inserts an item for `key` with `value` and `[width, height]`.
    ///
    /// Returns:
    /// - `Ok(Some(value))`: `key` already exists, `value` was not updated but returned,
    /// - `Ok(None)`: item has been inserted,
    /// - `Err((key, value))`: allocation failed.
    ///
    /// You ***MUST*** call this function for all items to be used in the current frame.
    ///
    /// If allocation fails, call [`Self::clear()`] before the next frame, or try a larger
    /// allocator.
    fn insert(&mut self, key: K, value: V, [width, height]: [u32; 2]) -> Result<Option<V>, (K, V)> {
        let [main, cross] = A::flip(width, height);

        // Check dimensions
        if !(main <= self.main() && cross <= self.cross()) {
            return Err((key, value));
        }

        self.try_insert(key, value, [main, cross]);

        Ok(None)
    }

    /// Clears the allocator.
    fn clear(&mut self) {
        self.clear_and_resize(self.width, self.height, self.bucket);
    }

    /// Clears and resizes the allocator.
    fn clear_and_resize(&mut self, width: u32, height: u32, bucket: u32) {
        assert!(width != 0 && height != 0 && bucket != 0);

        let main = A::main(width, height);
        let bucket = bucket.min(main);

        self.bucket = bucket;
        self.width = width;
        self.height = height;
        self.buckets.clear_and_resize(bucket, main);
        self.items.clear();
        self.frame = 0;
    }
}

enum Fits {
    Existing { b: u32, s: u32 },
    New { b: u32, position: u32 },
}

/// Private.
impl<A: Axis, K: Eq + Hash, V> Allocator<A, K, V> {
    fn main(&self) -> u32 {
        self.bucket
    }

    fn cross(&self) -> u32 {
        A::cross(self.width, self.height)
    }

    /// Checks a bucket for insertion.
    fn check_bucket(&self, b: u32, [main, cross]: [u32; 2]) -> Option<Fits> {
        let bucket = &self.buckets[b];
        let (closed_shelves, open_shelf) = bucket
            .shelves
            .split_last()
            .map(|(open_shelf, closed_shelves)| (closed_shelves, Some(open_shelf)))
            .unwrap_or_default();

        for (s, shelf) in closed_shelves.iter().enumerate() {
            let s = s as u32;

            // Fits in closed shelf?
            if (main <= bucket.size - shelf.main) && (cross <= shelf.cross) {
                return Some(Fits::Existing { b, s });
            }
        }

        let position = if let Some(shelf) = open_shelf {
            let s = closed_shelves.len() as u32;

            // Fits in open shelf?
            if (main <= bucket.size - shelf.main) && (cross <= self.cross() - shelf.position) {
                return Some(Fits::Existing { b, s });
            }

            shelf.position + shelf.cross
        } else {
            0
        };

        // Fits in new shelf?
        if (main <= bucket.size) && (cross <= self.cross() - position) {
            return Some(Fits::New { b, position });
        }

        None
    }

    /// Tries to insert an item.
    fn try_insert(&mut self, key: K, value: V, [main, cross]: [u32; 2]) -> Result<(), (K, V)> {
        let mut fits = None;

        for b in self.buckets.newest_to_oldest() {
            fits = self.check_bucket(b, [main, cross]);

            if fits.is_some() {
                break;
            }
        }

        let fits = if let Some(fits) = fits {
            fits
        } else {
            return Err((key, value));
        };

        match fits {
            Fits::Existing { b, s } => {
                self.insert_in_existing_shelf(b, s, key, value, [main, cross]);
            }
            Fits::New { b, position } => {
                self.insert_in_new_shelf(b, position, key, value, [main, cross]);
            }
        }

        Ok(())
    }

    fn insert_in_existing_shelf(
        &mut self,
        b: u32,
        s: u32,
        key: K,
        value: V,
        [main, cross]: [u32; 2],
    ) {
        self.buckets.touch(b, self.frame);

        let bucket = &mut self.buckets[b];
        let bucket_position = bucket.position;
        let shelf = &mut bucket[s];

        self.items.insert(
            key,
            Item {
                bucket: b,
                shelf: s,
                main: bucket_position + shelf.main,
                cross: shelf.position,
                value,
            },
        );

        shelf.frame = self.frame;
        shelf.main += main;
        shelf.cross = shelf.cross.max(cross);
    }

    fn insert_in_new_shelf(
        &mut self,
        b: u32,
        position: u32,
        key: K,
        value: V,
        [main, cross]: [u32; 2],
    ) {
        self.buckets.touch(b, self.frame);

        let bucket = &mut self.buckets[b];
        let bucket_position = bucket.position;

        self.items.insert(
            key,
            Item {
                bucket: b,
                shelf: bucket.shelves.len() as u32,
                main: bucket_position,
                cross: position,
                value,
            },
        );

        bucket.shelves.push(Shelf {
            frame: self.frame,
            position,
            main,
            cross,
        });
    }

    /// Tries to de-allocate the oldest bucket. Returns the index of the deallocated bucket.
    ///
    /// The deallocated bucket **MUST** be touched if this method succeeds.
    fn try_deallocate_bucket(&mut self, main: u32) -> Option<u32> {
        let mut oldest = None;

        for b in self.buckets.oldest_to_newest() {
            let bucket = &self.buckets[b];

            if bucket.frame != self.frame && main <= bucket.size {
                oldest = Some(b);
                break;
            }
        }

        let b = oldest?;
        let bucket = &mut self.buckets[b];

        // Remove shelves and items from this bucket
        bucket.shelves.clear();
        self.items.retain(|_, item| item.bucket != b);

        Some(b)
    }

    /// Tries to deallocate some oldest shelves. Returns the index of the partially deallocated
    /// bucket and the position of the new shelf.
    ///
    /// The deallocated bucket **MUST** be touched if this method succeeds.
    fn try_deallocate_shelves(&mut self, [main, cross]: [u32; 2]) -> Option<(u32, u32)> {
        let mut best: Option<(u32, u32, u32)> = None;

        for b in self.buckets.oldest_to_newest() {
            let bucket = &self.buckets[b];

            if main > bucket.size {
                continue;
            }

            let (s, shelf) = if let Some((s, shelf)) = bucket
                .shelves
                .iter()
                .enumerate()
                .rev()
                .take_while(|(_, shelf)| shelf.frame != self.frame)
                .last()
            {
                (s as u32, shelf)
            } else {
                continue;
            };

            let available = self.cross() - shelf.position;

            if cross > available {
                continue;
            }

            if let Some(best) = best.as_mut() {
                if available > self.cross() - best.2 {
                    *best = (b, s, shelf.position);
                }
            } else {
                best = Some((b, s, shelf.position));
            }
        }

        let (b, s, position) = best?;

        &mut self.buckets[b].shelves.truncate(s as usize);
        self.items
            .retain(|_, item| (item.bucket, item.shelf) != (b, s));

        Some((b, position))
    }
}
