//! Grids: state about every point in the game world.

use constants::*;

/// A location in 2d space.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Point {
    /// The X coordinate.
    pub x: usize,
    /// The Y coordnate.
    pub y: usize,
}

/// A grid representing some state of the world.
pub struct Grid<T> {
    pub grid: Box<[[T; WIDTH]; HEIGHT]>,
}

impl<T: Copy> Grid<T> {
    /// Construct a new grid with a zero value.
    pub fn new(zero: T) -> Grid<T> {
        Grid { grid: Box::new([[zero; WIDTH]; HEIGHT]) }
    }

    /// Look up a point in a grid.
    pub fn at(&self, p: Point) -> T {
        self.grid[p.y][p.x]
    }

    /// Set a point in a grid.
    pub fn set(&mut self, p: Point, val: T) {
        self.grid[p.y][p.x] = val
    }
}
