use std::{collections::HashMap, hash::Hash};

/// An [`Atlas`] item.
#[derive(Copy, Clone, Debug)]
pub struct Item<K> {
    bucket: usize,
    shelf: usize,
    x: usize,
    y: usize,
    prev: Option<K>,
    next: Option<K>,
}

impl<K> Item<K> {
    /// Returns the `x` position.
    pub fn x(&self) -> usize {
        self.x
    }

    /// Returns the `y` position.
    pub fn y(&self) -> usize {
        self.y
    }

    /// Returns the previous item's key.
    pub fn prev(&self) -> Option<&K> {
        self.prev.as_ref()
    }

    /// Returns the next item's key.
    pub fn next(&self) -> Option<&K> {
        self.next.as_ref()
    }
}

#[derive(Default, Debug)]
pub struct Shelf {
    items: usize,
    width: usize,
    height: usize,
}

#[derive(Default, Debug)]
pub struct Bucket {
    shelves: Vec<Shelf>,
}

/// A [`Shelf First Fit`](http://pds25.egloos.com/pds/201504/21/98/RectangleBinPack.pdf) [`Item`] atlas.
///
/// Divided in columns. Works as a cache. Has LRU linking between items.
#[derive(Debug)]
pub struct Atlas2<K: Clone + Eq + Hash> {
    column: usize,
    width: usize,
    height: usize,
    first: Option<K>,
    last: Option<K>,
    items: HashMap<K, Item<K>>,
    pub buckets: Vec<Bucket>,
}

impl<K: Clone + Eq + Hash> Atlas2<K> {
    /// Creates a new empty atlas with `width` and `height` and maximun `column` width.
    pub fn new(column: usize, width: usize, height: usize) -> Self {
        Self {
            column: column.min(width),
            width,
            height,
            first: None,
            last: None,
            items: Default::default(),
            buckets: Default::default(),
        }
    }

    /// Returns the maximum column size of the atlas.
    pub fn column(&self) -> usize {
        self.column
    }

    /// Returns the width of the atlas.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the atlas.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the first item's key.
    pub fn first(&self) -> Option<&K> {
        self.first.as_ref()
    }

    /// Returns the last item's key.
    pub fn last(&self) -> Option<&K> {
        self.last.as_ref()
    }

    /// Returns the item at `key`.
    pub fn get(&self, key: &K) -> Option<&Item<K>> {
        self.items.get(key)
    }

    /// Inserts and returns an item at `key` with `[width, height]`.
    ///
    /// Does not insert the same `key` twice. Does not remove items.
    pub fn insert(&mut self, key: K, [width, height]: [usize; 2]) -> Option<&Item<K>> {
        // Check dimensions
        if width > self.column || height > self.height {
            return None;
        }

        enum InCache<K> {
            True(Option<K>, Option<K>),
            False(Option<Item<K>>),
        }

        // Lookup cache
        let in_cache = if let Some((prev, next)) = self
            .items
            .get_mut(&key)
            .map(|item| (item.prev.clone(), item.next.clone()))
        {
            InCache::True(prev, next)
        }
        // Find free space
        else {
            let mut item = None;
            let mut bucket_x = 0;

            // Existing buckets
            'found_it: for (b, bucket) in self.buckets.iter_mut().enumerate() {
                let shelves = bucket.shelves.len();
                let bucket_width = self.column.min(self.width - bucket_x);

                // Fits in bucket
                if width <= bucket_width {
                    let mut shelf_y = 0;

                    // Closed shelves
                    for (s, shelf) in bucket.shelves.iter_mut().rev().skip(1).rev().enumerate() {
                        // Fits in shelf
                        if width <= bucket_width - shelf.width && height <= shelf.height {
                            item = Some(Item {
                                bucket: b,
                                shelf: s,
                                x: bucket_x + shelf.width,
                                y: shelf_y,
                                prev: None,
                                next: None,
                            });
                            shelf.items += 1;
                            shelf.width += width;
                            break 'found_it;
                        }

                        shelf_y += shelf.height;
                    }

                    // Open shelf
                    if let Some(shelf) = bucket.shelves.last_mut() {
                        // Fits in shelf
                        if width <= bucket_width - shelf.width && height <= self.height - shelf_y {
                            item = Some(Item {
                                bucket: b,
                                shelf: shelves - 1,
                                x: bucket_x + shelf.width,
                                y: shelf_y,
                                prev: None,
                                next: None,
                            });
                            shelf.items += 1;
                            shelf.width += width;
                            shelf.height = shelf.height.max(height);
                            break 'found_it;
                        }

                        shelf_y += shelf.height;
                    }

                    // Fits in new shelf
                    if height <= self.height - shelf_y {
                        item = Some(Item {
                            bucket: b,
                            shelf: shelves,
                            x: bucket_x,
                            y: shelf_y,
                            prev: None,
                            next: None,
                        });
                        bucket.shelves.push(Shelf {
                            items: 1,
                            width,
                            height,
                        });
                        break 'found_it;
                    }
                }

                bucket_x += bucket_width;
            }

            if item.is_none() {
                // Fits in new bucket
                if width < self.width - bucket_x {
                    item = Some(Item {
                        bucket: self.buckets.len(),
                        shelf: 0,
                        x: bucket_x,
                        y: 0,
                        prev: None,
                        next: None,
                    });
                    self.buckets.push(Bucket {
                        shelves: vec![Shelf {
                            items: 1,
                            width,
                            height,
                        }],
                    });
                }
            }

            InCache::False(item)
        };

        // Reorder items
        match in_cache {
            InCache::True(prev, next) => {
                if let Some(next) = next {
                    if let Some(prev) = prev {
                        self.items.get_mut(&prev).unwrap().next = Some(next.clone());
                        self.items.get_mut(&next).unwrap().next = Some(prev);
                    } else {
                        self.items.get_mut(&next).unwrap().prev = None;
                        self.first = Some(next);
                    }

                    self.last
                        .as_ref()
                        .map(|last| self.items.get_mut(last).unwrap().next = Some(key.clone()));

                    let item = self.items.get_mut(&key).unwrap();
                    item.prev = std::mem::replace(&mut self.last, Some(key.clone()));
                    item.next = None;

                    Some(item)
                } else {
                    self.items.get(&key)
                }
            }
            InCache::False(Some(mut item)) => {
                self.first.get_or_insert_with(|| key.clone());
                self.last
                    .as_ref()
                    .map(|last| self.items.get_mut(last).unwrap().next = Some(key.clone()));

                item.prev = std::mem::replace(&mut self.last, Some(key.clone()));

                Some(self.items.entry(key).or_insert(item))
            }
            InCache::False(None) => None,
        }
    }

    /// Removes and returns the item at `key`.
    pub fn remove(&mut self, key: &K) -> Option<Item<K>> {
        // Remove from HashMap
        let item = if let Some(item) = self.items.remove(key) {
            item
        } else {
            return None;
        };

        // Re-link prev and next
        match (&item.prev, &item.next) {
            // It was the only item in the Atlas
            (None, None) => {
                debug_assert!(self.first.as_ref() == Some(key));
                debug_assert!(self.last.as_ref() == Some(key));
                self.first = None;
                self.last = None;
            }
            // It was the first item in the Atlas
            (None, Some(next)) => {
                debug_assert!(self.first.as_ref() == Some(key));
                self.first = Some(next.clone());
                {
                    let next = self.items.get_mut(next).unwrap();
                    debug_assert!(next.prev.as_ref() == Some(key));
                    next.prev = None;
                }
            }
            // It was the last item in the Atlas
            (Some(prev), None) => {
                debug_assert!(self.last.as_ref() == Some(key));
                {
                    let prev = self.items.get_mut(prev).unwrap();
                    debug_assert!(prev.next.as_ref() == Some(key));
                    prev.next = None;
                }
                self.last = Some(prev.clone());
            }
            // It was neither first nor last in the Atlas
            (Some(prev), Some(next)) => {
                {
                    let prev = self.items.get_mut(prev).unwrap();
                    debug_assert!(prev.next.as_ref() == Some(key));
                    prev.next = Some(next.clone());
                }
                {
                    let next = self.items.get_mut(next).unwrap();
                    debug_assert!(next.prev.as_ref() == Some(key));
                    next.prev = Some(prev.clone());
                }
            }
        }

        // Decrement shelf's item count
        let bucket = self.buckets.get_mut(item.bucket).unwrap();
        let shelf = bucket.shelves.get_mut(item.shelf).unwrap();
        debug_assert!(shelf.items > 0);
        shelf.items -= 1;

        // Reclaim empty open shelves
        for i in (0..bucket.shelves.len()).rev() {
            if bucket.shelves[i].items == 0 {
                bucket.shelves.pop();
            } else {
                break;
            }
        }

        Some(item)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Atlas                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

const ZERO: u8 = 0;

/// A `Shelf Next Fit` Atlas.
#[derive(Debug)]
pub struct Atlas<K: Eq + Hash> {
    /// The image data.
    data: Vec<u8>,
    /// The width of the atlas.
    width: usize,
    /// The positions of the images.
    positions: HashMap<K, [usize; 2]>,
    /// The current position in the current shelf.
    x: usize,
    /// The current shelf's position in the atlas.
    y: usize,
    /// The current shelf's height.
    h: usize,
}

impl<K: Eq + Hash> Atlas<K> {
    /// Creates a new empty `Atlas` with `width` bytes.
    pub fn new(width: usize) -> Self {
        Self {
            data: Default::default(),
            width,
            positions: Default::default(),
            x: 0,
            y: 0,
            h: 0,
        }
    }

    /// Returns the `Atlas`'s width, in bytes.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the `Atlas`'s current height, in bytes.
    pub fn height(&self) -> usize {
        self.y + self.h
    }

    /// Returns the position of `key` in the `Atlas`.
    pub fn get(&self, key: &K) -> Option<([usize; 2])> {
        self.positions.get(key).copied()
    }

    /// Returns the `Atlas`'s data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Inserts `data` at `[width, height]` with `key` in the `Atlas` and returns its position,
    /// or `None` if it is larger than the `Atlas`'s width.
    pub fn set(&mut self, key: K, [width, height]: [usize; 2], data: &[u8]) -> Option<[usize; 2]> {
        debug_assert!(width * height == data.len());
        debug_assert!(self.x <= self.width);
        debug_assert!(self.width * (self.y + self.h) == self.data.len());

        if let Some(coordinates) = self.positions.get(&key) {
            return Some(*coordinates);
        } else if width as usize > self.width {
            return None;
        }

        let atlas_is_empty = self.positions.is_empty();
        let width_fits_on_shelf = self.x + width <= self.width;
        let height_fits_on_shelf = height <= self.h;

        if atlas_is_empty || !width_fits_on_shelf {
            // Add shelf
            self.data
                .resize(self.data.len() + self.width * height, ZERO);

            self.x = 0;
            self.y += self.h;
            self.h = height;
        } else if !height_fits_on_shelf {
            // Resize shelf
            self.data
                .resize(self.data.len() + self.width * (height - self.h), ZERO);

            self.h = height;
        }

        // Copy data
        {
            let mut start = self.x + self.y * self.width;

            for row in data.chunks_exact(width) {
                self.data[start..start + width].copy_from_slice(row);
                start += self.width;
            }
        }

        let position = [self.x, self.y];

        self.x += width;
        self.h = self.h.max(height);
        self.positions.insert(key, position);

        Some(position)
    }

    /// Clears the `Altas`.
    pub fn clear(&mut self) {
        self.data.clear();
        self.positions.clear();
        self.x = 0;
        self.y = 0;
        self.h = 0;
    }
}
