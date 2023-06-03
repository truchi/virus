use std::collections::HashMap;
use std::hash::Hash;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Atlas                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An [`Atlas`] item.
#[derive(Copy, Clone, Debug)]
struct Item {
    /// The bucket index.
    bucket: u32,
    /// The X coordinate in the atlas.
    x: u32,
    /// The Y coordinate in the atlas.
    y: u32,
}

/// An [`Atlas`] bucket.
///
/// A (fixed-height) row in the atlas.
#[derive(Default, Debug)]
struct Bucket {
    /// The most recent frame that inserted in this bucket.
    frame: u32,
    /// The X coordinate of the current shelf.
    shelf_x: u32,
    /// The width of the widest item in the current shelf.
    shelf_width: u32,
    /// The occupied height of the current shelf.
    shelf_height: u32,
}

/// A [`Shelf Next Fit`](http://pds25.egloos.com/pds/201504/21/98/RectangleBinPack.pdf) atlas.
///
/// The atlas is divided in (horiontal) rows, which are divided in (vertical) shelves.
/// It works as a cache: an item is not re-inserted if its key is found in the atlas.
///
/// You ***MUST*** call [`Atlas::next_frame()`] at the beginning of a new frame,
/// and ***ONLY*** at the beginning of a new frame.
///
/// De-allocation happens automatically.
/// The row that has oldest data is reclaimed when space is needed.
#[derive(Debug)]
pub struct Atlas<K: Eq + Hash> {
    /// The bucket height of the atlas.
    row: u32,
    /// The width of the atlas.
    width: u32,
    /// The height of the atlas.
    height: u32,
    /// The items in the atlas.
    items: HashMap<K, Item>,
    /// The buckets of the atlas.
    buckets: Vec<Bucket>,
    /// The current frame.
    frame: u32,
}

impl<K: Eq + Hash> Atlas<K> {
    /// Creates a new empty atlas with `width` and `height` and `row` height.
    ///
    /// Last row may be shorter than `row`.
    /// Works best with small rows and large size.
    pub fn new(row: u32, width: u32, height: u32) -> Self {
        Self {
            row: row.min(height),
            width,
            height,
            items: Default::default(),
            buckets: Default::default(),
            frame: 0,
        }
    }

    /// Returns the row height of the atlas.
    pub fn row(&self) -> u32 {
        self.row
    }

    /// Returns the width of the atlas.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the atlas.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the position in the altas of the item for `key`.
    ///
    /// If this item is to be used in the current frame, you ***MUST*** call [`Atlas::insert()`].
    pub fn get(&self, key: &K) -> Option<[u32; 2]> {
        self.items.get(key).map(|item| [item.x, item.y])
    }

    /// Marks the beginning of a new frame.
    ///
    /// You ***MUST*** call this function at the beginning of a new frame,
    /// and ***ONLY*** at the beginning of a new frame.
    ///
    /// Clears the atlas when the underlying `u32` would overflow (2.2 years at 60pfs).
    pub fn next_frame(&mut self) {
        if let Some(frame) = self.frame.checked_add(1) {
            self.frame = frame;
        } else {
            self.clear();
        }
    }

    /// Inserts an item for `key` with `[width, height]` and returns its position in the atlas,
    /// or `None` if allocation fails.
    ///
    /// Re-uses the position if `key` is already present.
    /// De-allocates oldest items automatically, when needed.
    ///
    /// If allocation fails, call [`Atlas::clear()`] before the next frame, or try a larger atlas.
    pub fn insert(&mut self, mut key: K, [width, height]: [u32; 2]) -> Option<[u32; 2]> {
        // Check dimensions
        if width > self.width || height > self.row {
            return None;
        }

        // Lookup cache
        if let Some(item) = self.items.get(&key) {
            // Update bucket's frame
            self.buckets[item.bucket as usize].frame = self.frame;

            return Some([item.x, item.y]);
        }

        // Insert or deallocate oldest bucket and retry (or fail)
        loop {
            match self.try_insert(key, [width, height]) {
                Ok(item) => return Some([item.x, item.y]),
                Err(k) => {
                    self.try_deallocate()?;
                    key = k;
                }
            }
        }
    }

    /// Clears the atlas.
    pub fn clear(&mut self) {
        self.items.clear();
        self.buckets.clear();
        self.frame = 0;
    }
}

/// Private.
impl<K: Eq + Hash> Atlas<K> {
    /// Tries to insert an item in the atlas.
    fn try_insert(&mut self, key: K, [width, height]: [u32; 2]) -> Result<Item, K> {
        debug_assert!(width <= self.width);
        debug_assert!(height <= self.row);

        let mut bucket_y = 0;

        // Existing buckets
        for (b, bucket) in self.buckets.iter_mut().enumerate() {
            let bucket_height = self.row.min(self.height - bucket_y);

            // Fits in current shelf
            if (width <= self.width - bucket.shelf_x)
                && (height <= bucket_height - bucket.shelf_height)
            {
                let item = Item {
                    bucket: b as u32,
                    x: bucket.shelf_x,
                    y: bucket_y + bucket.shelf_height,
                };

                self.items.insert(key, item);
                bucket.frame = self.frame;
                bucket.shelf_width = bucket.shelf_width.max(width);
                bucket.shelf_height += height;

                return Ok(item);
            }

            // Fits in new shelf
            if (width <= self.width - bucket.shelf_x - bucket.shelf_width)
                && (height <= bucket_height)
            {
                let item = Item {
                    bucket: b as u32,
                    x: bucket.shelf_x + bucket.shelf_width,
                    y: bucket_y,
                };

                self.items.insert(key, item);
                bucket.frame = self.frame;
                bucket.shelf_x += bucket.shelf_width;
                bucket.shelf_width = width;
                bucket.shelf_height = height;

                return Ok(item);
            }

            bucket_y += bucket_height;
        }

        // Fits in new bucket
        if height <= self.height - bucket_y {
            let item = Item {
                bucket: self.buckets.len() as u32,
                x: 0,
                y: bucket_y,
            };

            self.items.insert(key, item);
            self.buckets.push(Bucket {
                frame: self.frame,
                shelf_x: 0,
                shelf_width: width,
                shelf_height: height,
            });

            return Ok(item);
        }

        Err(key)
    }

    /// Tries to de-allocate the oldest bucket in the atlas.
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
        self.items
            .retain(|key, item| item.bucket as usize != bucket);
        self.buckets[bucket] = Default::default();

        Some(bucket)
    }
}
