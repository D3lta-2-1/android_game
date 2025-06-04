use nalgebra::Vector2;
use std::ops::Range;

#[derive(Debug, Copy, Clone, Default)]
pub struct SquareMask {
    power_of_two: usize,
}

impl SquareMask {
    fn morton_encode(&self, pos: Vector2<isize>) -> usize {
        let half_size = 1 << (self.power_of_two - 1);
        let pos = pos + Vector2::new(half_size, half_size); // Offset to avoid zero index
        Self::part1by1(pos.x as usize) | (Self::part1by1(pos.y as usize) << 1)
    }

    /// Decode Morton index into coordinates (x, y)
    fn morton_decode(&self, z: usize) -> Vector2<isize> {
        let half_size = 1 << (self.power_of_two - 1);
        Vector2::new(
            Self::compact1by1(z) as isize,
            Self::compact1by1(z >> 1) as isize,
        ) - Vector2::new(half_size, half_size)
    }

    pub fn index_to_pos(&self, z: usize) -> Vector2<isize> {
        self.morton_decode(z)
    }

    pub fn get<T: Copy>(&self, slice: &[T], pos: Vector2<isize>, default: T) -> T {
        let safe_range = self.range();
        if safe_range.contains(&pos.x) && safe_range.contains(&pos.y) {
            slice[self.morton_encode(pos)]
        } else {
            default
        }
    }

    pub fn set<T>(&self, slice: &mut [T], pos: Vector2<isize>, muter: impl FnOnce(&mut T)) {
        let safe_range = self.range();
        if safe_range.contains(&pos.x) && safe_range.contains(&pos.y) {
            muter(&mut slice[self.morton_encode(pos)]);
        }
    }

    /// Interleave bits of x and y (morton_encode)
    fn part1by1(mut n: usize) -> usize {
        n &= 0x0000ffff;
        n = (n | (n << 8)) & 0x00ff00ff;
        n = (n | (n << 4)) & 0x0f0f0f0f;
        n = (n | (n << 2)) & 0x33333333;
        n = (n | (n << 1)) & 0x55555555;
        n
    }

    /// Compact bits of n into a single number (morton_decode)
    fn compact1by1(mut n: usize) -> usize {
        n &= 0x55555555;
        n = (n | (n >> 1)) & 0x33333333;
        n = (n | (n >> 2)) & 0x0f0f0f0f;
        n = (n | (n >> 4)) & 0x00ff00ff;
        n = (n | (n >> 8)) & 0x0000ffff;
        n
    }

    pub fn new(size: usize) -> Self {
        Self { power_of_two: size }
    }

    pub fn create_grid<T: Clone>(&self, t: T) -> Vec<T> {
        self.index_range().map(|_| t.clone()).collect()
    }

    pub fn range(&self) -> Range<isize> {
        let half_size = 1 << (self.power_of_two - 1);
        -half_size..half_size
    }

    pub fn size(&self) -> usize {
        1 << self.power_of_two
    }

    pub fn index_range(&self) -> Range<usize> {
        0..self.size() * self.size()
    }
}
