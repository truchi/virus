use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Item                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An [`Atlas`] item.
#[derive(Copy, Clone, Debug)]
struct Item<V> {
    /// The top coordinate of the item in the atlas.
    top: u32,
    /// The left coordinate of the item in the atlas.
    left: u32,
    /// The value associated with the item.
    value: V,
}

impl<V> Item<V> {
    /// Returns the position of the item.
    fn position(&self) -> Position {
        debug_assert!(i32::try_from(self.top).is_ok());
        debug_assert!(i32::try_from(self.left).is_ok());

        Position {
            top: self.top as i32,
            left: self.left as i32,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Shelf                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An [`Atlas`] shelf.
#[derive(Copy, Clone, Debug)]
struct Shelf {
    /// The occupied width of the shelf.
    width: u32,
    /// The height of the largest item in the shelf.
    height: u32,
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
    bins: Vec<Vec<Shelf>>,
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
    pub fn get(&self, key: &K) -> Option<(Position, &V)> {
        self.items
            .get(&key)
            .map(|item| (item.position(), &item.value))
    }

    /// Inserts an item for `key` with `([width, height], value)` provided through `f`
    /// (only called when the item does not exist already).
    ///
    /// If allocation fails, call [`Self::clear()`] before the next frame, or try a larger
    /// atlas.
    //
    // TODO: Take a function returning the image data.
    pub fn insert(
        &mut self,
        queue: &Queue,
        key: K,
        value: V,
        size: Size,
        bytes: &[u8],
    ) -> Result<(Position, &V), AtlasError> {
        if self.items.contains_key(&key) {
            return Err(AtlasError::KeyExists);
        }

        self.try_insert(&key, value, size)?;

        let item = self.items.get(&key).unwrap();
        self.write(queue, (item.position(), size).into(), bytes);

        Ok((item.position(), &item.value))
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
    fn try_insert(
        &mut self,
        key: &K,
        value: V,
        Size { width, height }: Size,
    ) -> Result<(), AtlasError> {
        if !((width <= self.bin_width) && (height <= self.texture.height())) {
            return Err(AtlasError::WontFit);
        }

        let mut bin_left = 0;

        for bin in &mut self.bins {
            let bin_width = self.bin_width.min(self.texture.width() - bin_left);
            let mut shelf_top = 0;

            if let Some((open, closeds)) = bin.split_last_mut() {
                for closed in closeds {
                    // Fits in closed shelf?
                    if (width <= bin_width - closed.width) && (height <= closed.height) {
                        self.items.insert(
                            key.clone(),
                            Item {
                                top: shelf_top,
                                left: bin_left + closed.width,
                                value,
                            },
                        );
                        closed.width += width;

                        return Ok(());
                    }

                    shelf_top += closed.height;
                }

                // Fits in open shelf?
                if (width <= bin_width - open.width)
                    && (height <= self.texture.height() - shelf_top)
                {
                    self.items.insert(
                        key.clone(),
                        Item {
                            top: shelf_top,
                            left: bin_left + open.width,
                            value,
                        },
                    );
                    open.width += width;
                    open.height = open.height.max(height);

                    return Ok(());
                }

                shelf_top += open.height;
            }

            // Fits in new shelf?
            if (width <= bin_width) && (height <= self.texture.height() - shelf_top) {
                self.items.insert(
                    key.clone(),
                    Item {
                        top: shelf_top,
                        left: bin_left,
                        value,
                    },
                );
                bin.push(Shelf { width, height });

                return Ok(());
            }

            bin_left += bin_width;
        }

        // Fits in new bin?
        if width <= self.bin_width.min(self.texture.width() - bin_left) {
            self.items.insert(
                key.clone(),
                Item {
                    top: 0,
                    left: bin_left,
                    value,
                },
            );
            self.bins.push(vec![Shelf { width, height }]);

            return Ok(());
        }

        Err(AtlasError::OutOfSpace)
    }

    /// Writes `data` in texture.
    fn write(&self, queue: &Queue, rectangle: Rectangle, data: &[u8]) {
        let Rectangle {
            top,
            left,
            width,
            height,
        } = rectangle;

        queue.write_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: left as u32,
                    y: top as u32,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            &data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.texture.format().components() as u32 * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}
