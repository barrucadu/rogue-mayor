//! Utility functions.

use grid::*;
use mobiles::*;
use std::collections::BTreeMap;
use types::*;

/// Generate the includive range [from..to], which can be descending
/// or ascending.
pub fn inclusive_range(from: i8, to: i8) -> Vec<i8> {
    let mut out = Vec::new();
    let range = from.abs() as usize - to.abs() as usize + 1;
    for i in 0..range {
        if from > to {
            out.push(from - i as i8);
        } else {
            out.push(from + i as i8);
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
pub fn is_occupied(pos: Point, mobs: &BTreeMap<Point, Mobile>, _: &World) -> bool {
    mobs.get(&pos).is_some()
}
