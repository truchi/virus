// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Allocator                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub trait Allocator<K, V, B> {
    /// Creates a new empty allocator with `width` and `height`, and `bucket` cross-axis size.
    fn new(width: u32, height: u32, bucket: B) -> Self;

    /// Returns the width.
    fn width(&self) -> u32;

    /// Returns the height.
    fn height(&self) -> u32;

    /// Returns the bucket height.
    fn row(&self) -> u32;

    /// Returns the number of items.
    fn len(&self) -> usize;

    /// Returns the `[x, y]` position and value of the item for `key`.
    ///
    /// If this item is to be used in the current frame, you ***MUST*** call [`Self::insert()`].
    fn get(&self, key: &K) -> Option<([u32; 2], &V)>;

    /// Marks the beginning of a new frame.
    ///
    /// You ***MUST*** call this function inbewteen frames, and ***ONLY*** inbewteen frames,
    /// ***UNLESS*** you clear inbetween frames.
    ///
    /// Clears the allocator when the underlying `u32` would overflow (2.2 years at 60pfs).
    fn next_frame(&mut self);

    /// Inserts an item for `key` with `value` and `[width, height]`.
    ///
    /// Returns:
    /// - `Ok(Some(value))`: `key` already exists, `value` was not updated but returned,
    /// - `Ok(None)`: item is inserted,
    /// - `Err((key, value))`: allocation failed.
    ///
    /// You ***MUST*** call this function for all items to be used in the current frame.
    ///
    /// If allocation fails, call [`Self::clear()`] before the next frame, or try a larger
    /// allocator.
    fn insert(&mut self, key: K, value: V, size: [u32; 2]) -> Result<Option<V>, (K, V)>;

    /// Clears the allocator.
    fn clear(&mut self);

    /// Clears and resizes the allocator.
    fn clear_and_resize(&mut self, width: u32, height: u32, bucket: B);
}
