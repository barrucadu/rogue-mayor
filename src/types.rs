//! All the types. This is just a placeholder module as things get implemented and spread out into
//! their own modules.

use grid::*;

/// A command from the user.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Command {
    /// Terminate.
    Quit,
    /// Render the UI without advancing a turn.
    Render,
    /// Advance one turn without doing any user action.
    Skip,
}

/// The state of the vsible map and the larger game world.
pub struct World {
    /// Occupancy. This is a very simple temporary representation just to test the heatmaps.
    pub occupied: Grid<bool>,
}

impl World {
    /// Construct a new world.
    pub fn new() -> World {
        World { occupied: Grid::new(false) }
    }

    /// Do a turn.
    pub fn step(&mut self) {}
}
