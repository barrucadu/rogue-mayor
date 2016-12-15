//! Utility functions.

use grid::*;
use mobiles::*;
use std::collections::BTreeMap;
use types::*;

/// Generate the includive range [from..to], which can be descending
/// or ascending.
pub fn inclusive_range(from: i8, to: i8) -> Vec<i8> {
    let mut out = Vec::new();
    let range = 1 + if from < to { to - from } else { from - to };
    for i in 0..range {
        if from > to {
            out.push(from - i);
        } else {
            out.push(from + i);
        }
    }
    out
}

/// Perform a saturating addition or subtraction.
pub fn signed_add(u: usize, s: i8) -> usize {
    if s < 0 {
        u.saturating_sub(s.abs() as usize)
    } else {
        u.saturating_add(s.abs() as usize)
    }
}

/// Check if a position is occupied.
pub fn is_occupied(pos: Point, mobs: &BTreeMap<Point, Mobile>, world: &World) -> bool {
    mobs.get(&pos).is_some() || world.occupied.at(pos)
}
