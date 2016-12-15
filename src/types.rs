//! All the types. This is just a placeholder module as things get implemented and spread out into
//! their own modules.

use dijkstra_map::*;
use mobiles::Mobile;
use std::collections::BTreeMap;

/// A location in 2d space.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Point {
    /// The X coordinate.
    pub x: usize,
    /// The Y coordnate.
    pub y: usize,
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
    fn render(&mut self, &BTreeMap<Point, Mobile>, &Maps, &World);

    /// Await input.
    fn input(&self) -> Command;
}

/// A basic user interface.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BasicUI {}

impl UI for BasicUI {
    fn render(&mut self, _: &BTreeMap<Point, Mobile>, _: &Maps, _: &World) {}
    fn input(&self) -> Command {
        Command::Quit
    }
}
