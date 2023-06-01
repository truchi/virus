use std::{collections::HashMap, hash::Hash};

const ZERO: u8 = 0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Atlas                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

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
