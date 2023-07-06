use super::allocator::Allocator;
use std::{collections::HashMap, hash::Hash};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Item                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A [`NextFitAllocator`] item.
#[derive(Copy, Clone, Debug)]
struct Item<V> {
    /// The bucket index.
    bucket: u32,
    /// The main-axis coordinate.
    main: u32,
    /// The cross-axis coordinate.
    cross: u32,
    /// The value.
    value: V,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Bucket                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A [`NextFitAllocator`] bucket.
///
/// A (fixed-size) cross-axis slice of the atlas.
#[derive(Default, Debug)]
struct Bucket {
    /// The most recent frame that inserted in this bucket.
    frame: u32,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        NextFitAllocator                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A next-fit atlas allocator (bucketed, with deallocation).
pub struct NextFitAllocator<K: Eq + Hash, V> {
    /// The bucket cross-axis size.
    bucket: u32,
    /// The width of the atlas.
    width: u32,
    /// The height of the atlas.
    height: u32,
    /// The items in the atlas.
    items: HashMap<K, Item<V>>,
    /// The buckets of the atlas.
    buckets: Vec<Bucket>,
    /// The current frame.
    frame: u32,
    /// The X coordinate.
    /// The cross-axis offset of the current shelf.
    shelf_offset: u32,
    /// The width of the widest item in the current shelf.
    shelf_width: u32,
    /// The occupied height of the current shelf.
    shelf_height: u32,
}

impl<K: Eq + Hash, V> Allocator<K, V, u32> for NextFitAllocator<K, V> {
    fn new(width: u32, height: u32, bucket: u32) -> Self {
        Self {
            bucket: bucket.min(height),
            width,
            height,
            items: Default::default(),
            buckets: Default::default(),
            frame: 0,
            shelf_offset: 0,
            shelf_width: 0,
            shelf_height: 0,
        }
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn row(&self) -> u32 {
        self.bucket
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn get(&self, key: &K) -> Option<([u32; 2], &V)> {
        self.items
            .get(key)
            .map(|item| ([item.cross, item.main], &item.value))
    }

    fn next_frame(&mut self) {
        if let Some(frame) = self.frame.checked_add(1) {
            self.frame = frame;
        } else {
            self.clear();
        }
    }

    fn insert(&mut self, key: K, value: V, [width, height]: [u32; 2]) -> Result<Option<V>, (K, V)> {
        // Check dimensions
        if width > self.width || height > self.bucket {
            return Err((key, value));
        }

        // Lookup cache
        if let Some(item) = self.items.get(&key) {
            // Update bucket's frame
            self.buckets[item.bucket as usize].frame = self.frame;

            return Ok(Some(value));
        }

        // Insert or deallocate oldest bucket and retry (or fail)
        let mut key = key;
        let mut value = value;
        loop {
            (key, value) = match self.try_insert(key, value, [width, height]) {
                Ok(()) => return Ok(None),
                Err((k, v)) => (k, v),
            };

            if self.try_deallocate().is_none() {
                return Err((key, value));
            }
        }
    }

    fn clear(&mut self) {
        self.items.clear();
        self.buckets.clear();
        self.frame = 0;
    }

    fn clear_and_resize(&mut self, width: u32, height: u32, bucket: u32) {
        self.clear();

        self.bucket = bucket.min(height);
        self.width = width;
        self.height = height;
    }
}

/// Private.
impl<K: Eq + Hash, V> NextFitAllocator<K, V> {
    /// Tries to insert an item.
    fn try_insert(&mut self, key: K, value: V, [width, height]: [u32; 2]) -> Result<(), (K, V)> {
        // /!\ Important
        debug_assert!(width <= self.width);
        debug_assert!(height <= self.bucket);

        let buckets = self.buckets.len() as u32;

        let bucket_y = if let Some(bucket) = self.buckets.last_mut() {
            let bucket_y = buckets * self.bucket;
            let bucket_height = self.bucket.min(self.height - bucket_y);

            // Fits in current shelf
            if (width <= self.width - self.shelf_offset)
                && (height <= bucket_height - self.shelf_height)
            {
                self.items.insert(
                    key,
                    Item {
                        bucket: buckets - 1,
                        cross: self.shelf_offset,
                        main: bucket_y + self.shelf_height,
                        value,
                    },
                );
                bucket.frame = self.frame;
                self.shelf_width = self.shelf_width.max(width);
                self.shelf_height += height;

                return Ok(());
            }

            // Fits in new shelf
            if (width <= self.width - self.shelf_offset - self.shelf_width)
                && (height <= bucket_height)
            {
                self.items.insert(
                    key,
                    Item {
                        bucket: buckets - 1,
                        cross: self.shelf_offset + self.shelf_width,
                        main: bucket_y,
                        value,
                    },
                );
                bucket.frame = self.frame;
                self.shelf_offset += self.shelf_width;
                self.shelf_width = width;
                self.shelf_height = height;

                return Ok(());
            }

            bucket_y + bucket_height
        } else {
            0
        };

        // Fits in new bucket
        if height <= self.height - bucket_y {
            self.items.insert(
                key,
                Item {
                    bucket: buckets,
                    cross: 0,
                    main: bucket_y,
                    value,
                },
            );
            self.buckets.push(Bucket { frame: self.frame });
            self.shelf_offset = 0;
            self.shelf_width = width;
            self.shelf_height = height;

            return Ok(());
        }

        Err((key, value))
    }

    /// Tries to de-allocate the oldest bucket.
    fn try_deallocate(&mut self) -> Option<usize> {
        // Oldest (but non-current frame) bucket
        let bucket = self
            .buckets
            .iter()
            .enumerate()
            .min_by_key(|(_, bucket)| bucket.frame)
            .filter(|(_, bucket)| bucket.frame != self.frame)?
            .0;

        // Remove items from this bucket
        self.items.retain(|_, item| item.bucket as usize != bucket);
        self.buckets[bucket] = Default::default();

        Some(bucket)
    }
}
