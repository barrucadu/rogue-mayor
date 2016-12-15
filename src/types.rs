//! All the types. This is just a placeholder module as things get implemented and spread out into
//! their own modules.

use grid::*;
use std::collections::VecDeque;

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
    /// Message log.
    pub messages: VecDeque<Message>,
}

impl World {
    /// Construct a new world.
    pub fn new() -> World {
        World {
            occupied: Grid::new(false),
            messages: VecDeque::new(),
        }
    }

    /// Log a new message.
    pub fn log(&mut self, msg: Message) {
        self.messages.push_front(msg);
    }

    /// Do a turn.
    pub fn step(&mut self) {}
}

/// A message consists of some text and an optional location. The UI intelligently handle the
/// location (eg, jump-to-location).
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Message {
    /// The message text.
    pub msg: String,
    /// The optional location.
    pub loc: Option<Point>,
}
