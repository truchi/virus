use std::collections::HashMap;
use std::hash::Hash;

/// An [`Atlas`] item.
#[derive(Copy, Clone, Debug)]
struct Item {
    frame: u32,
    bucket: u32,
    x: u32,
    y: u32,
}

#[derive(Default, Debug)]
struct Bucket {
    frame: u32,
    shelf_x: u32,
    shelf_width: u32,
    shelf_height: u32,
}

/// A [`Shelf Next Fit`](http://pds25.egloos.com/pds/201504/21/98/RectangleBinPack.pdf) atlas.
#[derive(Debug)]
pub struct Atlas<K: Eq + Hash> {
    row: u32,
    width: u32,
    height: u32,
    items: HashMap<K, Item>,
    buckets: Vec<Bucket>,
    frame: u32,
}

impl<K: Eq + Hash> Atlas<K> {
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

    pub fn next_frame(&mut self) {
        if let Some(frame) = self.frame.checked_add(1) {
            self.frame = frame;
        } else {
            self.clear();
        }
    }

    // TODO remove
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.items.keys()
    }

    pub fn get(&self, key: &K) -> Option<[u32; 2]> {
        self.items.get(key).map(|item| [item.x, item.y])
    }

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

    pub fn clear(&mut self) {
        self.items.clear();
        self.buckets.clear();
        self.frame = 0;
    }
}

/// Private.
impl<K: Eq + Hash> Atlas<K> {
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
                    frame: self.frame,
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
                    frame: self.frame,
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
                frame: self.frame,
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
