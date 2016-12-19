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
    mobs.get(&pos).is_some() || world.statics.at(pos).map_or(false, |s| s.is_impassable)
}

/// Check if a tile can be seen from another.
pub fn can_see(start: Point, end: Point, world: &World) -> bool {
    let mut pos = start;

    let (dx, ix) = make_delta(pos.x, end.x);
    let (dy, iy) = make_delta(pos.y, end.y);

    let (inc_x, inc_y, mut err, err_inc, err_dec, corr_x, corr_y, counter) = if dx > dy {
        (if ix { 1 } else { -1 },
         0,
         dy as i32 * 2 - dx as i32,
         dy as i32 * 2,
         dx as i32 * 2,
         0,
         if iy { 1 } else { -1 },
         dx)
    } else {
        (0,
         if iy { 1 } else { -1 },
         dx as i32 * 2 - dy as i32,
         dx as i32 * 2,
         dy as i32 * 2,
         if ix { 1 } else { -1 },
         0,
         dy)
    };

    for _ in 0..counter {
        if err >= 0 {
            err -= err_dec;
            pos.x = signed_add(pos.x, corr_x);
            pos.y = signed_add(pos.y, corr_y);
        }
        err += err_inc;
        pos.x = signed_add(pos.x, inc_x);
        pos.y = signed_add(pos.y, inc_y);
        if world.statics.at(pos).map_or(false, |s| s.is_opaque) {
            return false;
        }
    }

    true
}

/// Take the delta between two values, and return the gradient.
fn make_delta(start: usize, end: usize) -> (usize, bool) {
    if start < end {
        (end - start, true)
    } else {
        (start - end, false)
    }
}
