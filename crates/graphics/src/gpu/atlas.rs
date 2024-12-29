use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Item                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An [`Atlas`] item.
#[derive(Copy, Clone, Debug)]
pub struct Item<V> {
    /// The position of the item in the atlas.
    pub position: Position,
    /// The value associated with the item.
    pub value: V,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Atlas                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Error type for [`Atlas::insert()`].
#[derive(Copy, Clone, Debug)]
pub enum AtlasError {
    /// The item already exists in the atlas.
    KeyExists,
    /// The item is too big for the atlas' remaining space. Clear the atlas.
    OutOfSpace,
    /// The item is too big for the atlas' width/height/bin dimensions. Resize the atlas.
    WontFit,
}

/// An atlas.
#[derive(Debug)]
pub struct Atlas<K: Clone + Eq + Hash, V> {
    /// The width of bins (last may be smaller).
    bin_width: u32,
    /// The bins of the atlas.
    bins: Vec<Vec<Size>>,
    /// The items in the atlas.
    items: HashMap<K, Item<V>>,
    /// The GPU texture.
    texture: Texture,
}

impl<K: Clone + Eq + Hash, V> Atlas<K, V> {
    /// Creates a new empty atlas into `texture` with `bin_witdth`.
    pub fn new(texture: Texture, bin_width: u32) -> Self {
        Self {
            bin_width: bin_width.min(texture.width()),
            bins: Default::default(),
            items: Default::default(),
            texture,
        }
    }

    /// Returns the GPU texture.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Returns the position and value of the item for `key`.
    pub fn get(&self, key: &K) -> Option<&Item<V>> {
        self.items.get(&key)
    }

    /// Inserts an item for `key` with `size` and `value`.
    ///
    /// If allocation fails, call [`Self::clear()`] before the next frame, or try a larger
    /// atlas.
    pub fn insert(
        &mut self,
        queue: &Queue,
        key: K,
        value: V,
        size: Size,
        bytes: &[u8],
    ) -> Result<&Item<V>, AtlasError> {
        if self.items.contains_key(&key) {
            return Err(AtlasError::KeyExists);
        }

        let item = {
            self.try_insert(&key, value, size)?;
            self.items.get(&key).unwrap()
        };

        self.write(queue, Rectangle::new(item.position, size), bytes);

        Ok(item)
    }

    /// Clears the atlas.
    pub fn clear(&mut self) {
        self.items.clear();
        self.bins.clear();
    }

    /// Clears and resizes the atlas.
    pub fn clear_and_resize(&mut self, texture: Texture, bin: u32) {
        self.clear();
        self.bin_width = bin.min(texture.width());
        self.texture = texture;
    }
}

/// Private.
impl<K: Clone + Eq + Hash, V> Atlas<K, V> {
    /// Tries to insert an item.
    fn try_insert(&mut self, key: &K, value: V, size: Size) -> Result<(), AtlasError> {
        if !(size.width <= self.bin_width && size.height <= self.texture.height()) {
            return Err(AtlasError::WontFit);
        }

        let mut bin_left = 0;

        for bin in &mut self.bins {
            let bin_width = self.bin_width.min(self.texture.width() - bin_left);
            let mut shelf_top = 0;

            if let Some((open, closeds)) = bin.split_last_mut() {
                for closed in closeds {
                    // Fits in closed shelf?
                    if (size.width <= bin_width - closed.width) && (size.height <= closed.height) {
                        self.items.insert(
                            key.clone(),
                            Item {
                                position: Position::new_u32(shelf_top, bin_left + closed.width),
                                value,
                            },
                        );
                        closed.width += size.width;

                        return Ok(());
                    }

                    shelf_top += closed.height;
                }

                // Fits in open shelf?
                if (size.width <= bin_width - open.width)
                    && (size.height <= self.texture.height() - shelf_top)
                {
                    self.items.insert(
                        key.clone(),
                        Item {
                            position: Position::new_u32(shelf_top, bin_left + open.width),
                            value,
                        },
                    );
                    open.width += size.width;
                    open.height = open.height.max(size.height);

                    return Ok(());
                }

                shelf_top += open.height;
            }

            // Fits in new shelf?
            if (size.width <= bin_width) && (size.height <= self.texture.height() - shelf_top) {
                self.items.insert(
                    key.clone(),
                    Item {
                        position: Position::new_u32(shelf_top, bin_left),
                        value,
                    },
                );
                bin.push(size);

                return Ok(());
            }

            bin_left += bin_width;
        }

        // Fits in new bin?
        if size.width <= self.bin_width.min(self.texture.width() - bin_left) {
            self.items.insert(
                key.clone(),
                Item {
                    position: Position::new_u32(0, bin_left),
                    value,
                },
            );
            self.bins.push(vec![size]);

            return Ok(());
        }

        Err(AtlasError::OutOfSpace)
    }

    /// Writes `data` in texture.
    fn write(&self, queue: &Queue, rectangle: Rectangle, data: &[u8]) {
        queue.write_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: rectangle.left as u32,
                    y: rectangle.top as u32,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            &data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.texture.format().components() as u32 * rectangle.width),
                rows_per_image: Some(rectangle.height),
            },
            Extent3d {
                width: rectangle.width,
                height: rectangle.height,
                depth_or_array_layers: 1,
            },
        );
    }
}
