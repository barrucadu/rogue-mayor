//! All the types. This is just a placeholder module as things get implemented and spread out into
//! their own modules.

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
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct World {
}

impl World {
    /// Do a turn.
    pub fn step(&mut self) {}
}
