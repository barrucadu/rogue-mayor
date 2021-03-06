//! Grids: state about every point in the game world.

use constants::*;
use std::fmt::{Debug, Error, Formatter};

/// A location in 2d space.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Point {
    /// The X coordinate.
    pub x: usize,
    /// The Y coordnate.
    pub y: usize,
}

impl Point {
    /// Offset one point by another.
    pub fn offset(&self, off: Point) -> Point {
        Point {
            x: self.x + off.x,
            y: self.y + off.y,
        }
    }
}

/// A grid representing some state of the world.
pub struct Grid<T> {
    /// The grid, as an array pf arrays.
    pub grid: Box<[[T; WIDTH]; HEIGHT]>,
}

impl<T> Debug for Grid<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(formatter, "Grid<T>")
    }
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
