use std::{collections::HashMap, hash::Hash, marker::PhantomData};
use virus_common::{Position, Rectangle, Size};
use wgpu::{Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, Texture, TextureAspect};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Axes                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Shelves axis.
pub trait Axis {
    /// Flips `a` and `b` when [`Vertical`].
    fn flip<T>(a: T, b: T) -> [T; 2];
}

/// Horizontal shelves.
#[derive(Copy, Clone, Debug)]
pub enum Horizontal {}

impl Axis for Horizontal {
    fn flip<T>(a: T, b: T) -> [T; 2] {
        [a, b]
    }
}

/// Vertical shelves.
#[derive(Copy, Clone, Debug)]
pub enum Vertical {}

impl Axis for Vertical {
    fn flip<T>(a: T, b: T) -> [T; 2] {
        [b, a]
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Item                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An [`Atlas`] item.
#[derive(Copy, Clone, Debug)]
struct Item<V> {
    /// The main-axis coordinate.
    main: u32,
    /// The cross-axis coordinate.
    cross: u32,
    /// The value.
    value: V,
}

impl<V> Item<V> {
    /// Returns the position of the item.
    fn position<A: Axis>(&self) -> Position {
        let [left, top] = A::flip(self.main as i32, self.cross as i32);

        Position { top, left }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Shelf                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An [`Atlas`] shelf.
#[derive(Copy, Clone, Debug)]
struct Shelf {
    /// The occupied main-axis size.
    main: u32,
    /// The cross-axis size of the largest item.
    cross: u32,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Atlas                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Error type for [`Atlas::insert()`].
#[derive(Copy, Clone, Debug)]
pub enum AtlasError {
    /// The item already exists in the atlas.
    KeyExists,
    /// The item is too big for the atlas' width/height/bin dimensions. Resize the atlas.
    WontFit,
    /// The item is too big for the atlas' remaining space. Clear the atlas.
    OutOfSpace,
}

/// An atlas.
#[derive(Debug)]
pub struct Atlas<A: Axis, K: Clone + Eq + Hash, V> {
    /// The bin main-axis size.
    bin: u32,
    /// The bins of the atlas.
    bins: Vec<Vec<Shelf>>,
    /// The items in the atlas.
    items: HashMap<K, Item<V>>,
    /// The GPU texture.
    texture: Texture,
    /// The shelves' axis.
    _axis: PhantomData<A>,
}

impl<A: Axis, K: Clone + Eq + Hash, V> Atlas<A, K, V> {
    /// Creates a new empty atlas into `texture` with `bin` main-axis size.
    pub fn new(texture: Texture, bin: u32) -> Self {
        let [main, _] = A::flip(texture.width(), texture.height());

        Self {
            bin: bin.min(main),
            bins: Default::default(),
            items: Default::default(),
            texture,
            _axis: PhantomData,
        }
    }

    /// Returns the position and value of the item for `key`.
    pub fn get(&self, key: &K) -> Option<(Position, &V)> {
        self.items
            .get(&key)
            .map(|item| (item.position::<A>(), &item.value))
    }

    /// Returns the GPU texture.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Inserts an item for `key` with `([width, height], value)` provided through `f`
    /// (only called when the item does not exist already).
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
    ) -> Result<(Position, &V), AtlasError> {
        // Check if key exists
        if self.items.contains_key(&key) {
            return Err(AtlasError::KeyExists);
        }

        let [main, cross] = A::flip(size.width, size.height);
        let [_, self_cross] = A::flip(self.texture.width(), self.texture.height());

        // Check dimensions
        if !(main <= self.bin && cross <= self_cross) {
            return Err(AtlasError::WontFit);
        }

        // Insert or fail
        self.try_insert(&key, value, [main, cross])?;

        // Lookup cache
        let item = self.items.get(&key).unwrap();
        let position = item.position::<A>();
        self.write(queue, position & size, bytes);

        Ok((position, &item.value))
    }

    /// Clears the atlas.
    pub fn clear(&mut self) {
        self.items.clear();
        self.bins.clear();
    }

    /// Clears and resizes the atlas.
    pub fn clear_and_resize(&mut self, texture: Texture, bin: u32) {
        let [main, _] = A::flip(texture.width(), texture.height());

        self.clear();
        self.bin = bin.min(main);
        self.texture = texture;
    }
}

/// Private.
impl<A: Axis, K: Clone + Eq + Hash, V> Atlas<A, K, V> {
    /// Tries to insert an item.
    fn try_insert(&mut self, key: &K, value: V, [main, cross]: [u32; 2]) -> Result<(), AtlasError> {
        let [self_main, self_cross] = A::flip(self.texture.width(), self.texture.height());

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
