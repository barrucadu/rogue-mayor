//! All the types. This is just a placeholder module as things get implemented and spread out into
//! their own modules.

use constants::*;
use std::collections::BTreeMap;
use std::fmt::{Debug, Error, Formatter};

/// A type of heatmap: each maptag has an associated map.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MapTag {
}

/// A location in 2d space.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Point {
    x: usize,
    y: usize,
}

/// Things which roam around in the world, like people and monsters.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Mobile {
}

impl Mobile {
    /// Do a turn.
    pub fn step(&self,
                _: Point,
                _: &mut BTreeMap<Point, Mobile>,
                _: &mut BTreeMap<MapTag, Map>,
                _: &mut World) {
    }
}

/// A heatmap.
#[derive(Copy)]
pub struct Map([[usize; WIDTH]; HEIGHT]);

impl Clone for Map {
    fn clone(&self) -> Map {
        *self
    }

    // Overwrite the provided array, rather than allocate a new one.
    fn clone_from(&mut self, source: &Map) {
        let Map(me) = *source;
        let Map(mut out) = *self;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                out[y][x] = me[y][x];
            }
        }
    }
}

impl Debug for Map {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let Map(me) = *self;
        let mut has_prior = false;

        try!(write!(formatter, "["));
        for row in me.iter() {
            // Prepend a comma if this isn't the first entry.
            if has_prior {
                try!(write!(formatter, ","));
            } else {
                has_prior = true;
            }

            // Output a single row.
            try!(formatter.debug_list().entries(row.iter()).finish());
        }
        write!(formatter, "]")
    }
}

/// A command from the user.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Command {
    /// Advance one turn without doing any user action.
    Skip,
    /// Terminate.
    Quit,
}

/// The state of the vsible map and the larger game world.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct World {
}

impl World {
    /// Do a turn.
    pub fn step(&mut self) {}
}

/// The UI. Implemented as a trait to allow for differing implementations.
pub trait UI {
    /// Render
    fn render(&mut self, &BTreeMap<Point, Mobile>, &BTreeMap<MapTag, Map>, &World);
}

/// A basic user interface.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BasicUI {}

impl UI for BasicUI {
    fn render(&mut self, _: &BTreeMap<Point, Mobile>, _: &BTreeMap<MapTag, Map>, _: &World) {}
}
