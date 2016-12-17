//! Static constants about the game world.

/// The width of the playable map.
pub const WIDTH: usize = 200;

/// The height of the playable map.
pub const HEIGHT: usize = 100;

/// The dijkstra map fleeing coefficient for cowards.
pub const COWARDICE_COEFF: f64 = -1.1;

/// The dijksra map fleeing coefficient for brave souls.
pub const BRAVERY_COEFF: f64 = -1.6;
