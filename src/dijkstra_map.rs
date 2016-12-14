//! Dijkstra maps.

///
/// See:
/// - http://www.roguebasin.com/index.php?title=The_Incredible_Power_of_Dijkstra_Maps
/// - http://www.roguebasin.com/index.php?title=Dijkstra_Maps_Visualized

use constants::*;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fmt::{Debug, Error, Formatter};
use std::usize;
use types::*;
use utils::*;

/// A type of heatmap: each maptag has an associated map.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MapTag {
}

/// A Dijkstra map, or heatmap.
#[derive(Copy)]
pub struct Map {
    /// Dijkstra map for approaching.
    pub approach: [[usize; WIDTH]; HEIGHT],
    /// Dijkstra map for fleeing, where the fleeing creature in question is not willing to take many
    /// risks to escape.. This is the approaching map multipled by a negative coefficient and
    /// rescanned to smooth out corners and the like.
    pub flee_cowardly: [[usize; WIDTH]; HEIGHT],
    /// Dijkstra map for fleeing, where the fleeing creature in question is willing to take more
    /// risks to escape. This is the approaching map multipled by a negative coefficient and
    /// rescanned to smooth out corners and the like.
    pub flee_bravely: [[usize; WIDTH]; HEIGHT],
}

impl Clone for Map {
    fn clone(&self) -> Map {
        *self
    }

    // Overwrite the provided array, rather than allocate a new one.
    fn clone_from(&mut self, source: &Map) {
        let Map { approach: me_a, flee_cowardly: me_fc, flee_bravely: me_fb } = *source;
        let Map { approach: mut out_a, flee_cowardly: mut out_fc, flee_bravely: mut out_fb } =
            *self;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                out_a[y][x] = me_a[y][x];
                out_fc[y][x] = me_fc[y][x];
                out_fb[y][x] = me_fb[y][x];
            }
        }
    }
}

impl Debug for Map {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        fn debug_array(formatter: &mut Formatter,
                       arr: [[usize; WIDTH]; HEIGHT])
                       -> Result<(), Error> {
            let mut has_prior = false;

            try!(write!(formatter, "["));
            for row in arr.iter() {
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

        let Map { approach: a, flee_cowardly: fc, flee_bravely: fb } = *self;
        try!(write!(formatter, "("));
        try!(debug_array(formatter, a));
        try!(write!(formatter, ","));
        try!(debug_array(formatter, fc));
        try!(write!(formatter, ","));
        try!(debug_array(formatter, fb));
        try!(write!(formatter, ")"));
        Ok(())
    }
}

/// Create a new map from the given goals.
pub fn new_map_from_goals(goals: Vec<Point>, world: &World) -> Map {
    let mut out = Map {
        approach: [[usize::MAX; WIDTH]; HEIGHT],
        flee_cowardly: [[usize::MAX; WIDTH]; HEIGHT],
        flee_bravely: [[usize::MAX; WIDTH]; HEIGHT],
    };

    // Make the goals all global minima.
    for goal in &goals {
        out.approach[goal.y][goal.x] = 0;
    }

    // Fill in the rest of the approach map.
    flood_fill(&mut out.approach, &goals, world);

    // Compute the fleeing maps and find their global minima.
    let mut minima: Vec<Point> = Vec::new();
    let mut minimal: usize = usize::MAX;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if out.approach[y][x] != usize::MAX {
                out.flee_cowardly[y][x] =
                    (out.approach[y][x] as f64 * COWARDICE_COEFF).round() as usize;
                out.flee_bravely[y][x] =
                    (out.approach[y][x] as f64 * BRAVERY_COEFF).round() as usize;

                if out.flee_cowardly[y][x] == minimal {
                    minima.push(Point { x: x, y: y });
                } else if out.flee_cowardly[y][x] < minimal {
                    minima = vec![Point { x: x, y: y }];
                    minimal = out.flee_cowardly[y][x];
                }
            }
        }
    }

    // Smooth the fleeing aps by flood filling from their minima.
    flood_fill(&mut out.flee_cowardly, &minima, world);
    flood_fill(&mut out.flee_bravely, &minima, world);

    out
}

/// Flood fill out from some points. When considering a new point, this behaves as follows:
/// - If the point is impassable, it keeps its current value.
/// - If the point is passable, it is assigned the value 1+cheapest neighbour.
///
/// This function assumes that the points given are the global minima, and may not perform properly
/// if that is not the case.
fn flood_fill(map: &mut [[usize; WIDTH]; HEIGHT], minima: &Vec<Point>, world: &World) {
    let mut queue: VecDeque<Point> = VecDeque::with_capacity(WIDTH * HEIGHT / 2);
    for m in minima {
        queue.push_back(*m);
    }

    // Flood fill.
    while let Some(pos) = queue.pop_front() {
        // Only consider permanent fixtures, not mobs.
        if !is_occupied(pos, &BTreeMap::new(), world) {
            // Compute the local minima ans also the adjacent tiles > the current value.
            let mut local_min = map[pos.y][pos.x];
            let mut adj = Vec::new();
            for dy in inclusive_range(-1, 1) {
                if (dy < 0 && pos.y == 0) || (dy > 0 && pos.y == HEIGHT - 1) {
                    continue;
                }
                let y = signed_add(pos.y, dy);
                for dx in inclusive_range(-1, 1) {
                    if (dx < 0 && pos.x == 0) || (dx > 0 && pos.x == WIDTH - 1) {
                        continue;
                    }
                    let x = signed_add(pos.x, dx);
                    if map[y][x] < local_min {
                        local_min = map[y][x];
                    } else if map[y][x] > map[pos.y][pos.x] {
                        adj.push(Point { x: x, y: y });
                    }
                }
            }

            // If this results in a change of weight, push all the adjacent tiles > the old value.
            if local_min + 1 < map[pos.y][pos.x] {
                map[pos.y][pos.x] = local_min + 1;
                for a in adj {
                    queue.push_back(a);
                }
            }
        }
    }
}
