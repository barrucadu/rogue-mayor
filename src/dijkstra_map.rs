//! Dijkstra maps.

///
/// See:
/// - http://www.roguebasin.com/index.php?title=The_Incredible_Power_of_Dijkstra_Maps
/// - http://www.roguebasin.com/index.php?title=Dijkstra_Maps_Visualized

use constants::*;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::f64;
use std::fmt::{Debug, Error, Formatter};
use types::*;
use utils::*;

/// A type of heatmap: each maptag has an associated map.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MapTag {
}

/// A Dijkstra map, or heatmap.
pub struct Map {
    /// The sources (the global minima of the approach map).
    pub sources: Vec<Point>,
    /// Dijkstra map for approaching.
    pub approach: [[f64; WIDTH]; HEIGHT],
    /// Dijkstra map for fleeing, where the fleeing creature in question is not willing to take many
    /// risks to escape.. This is the approaching map multipled by a negative coefficient and
    /// rescanned to smooth out corners and the like.
    pub flee_cowardly: [[f64; WIDTH]; HEIGHT],
    /// Dijkstra map for fleeing, where the fleeing creature in question is willing to take more
    /// risks to escape. This is the approaching map multipled by a negative coefficient and
    /// rescanned to smooth out corners and the like.
    pub flee_bravely: [[f64; WIDTH]; HEIGHT],
}

impl Clone for Map {
    fn clone(&self) -> Map {
        let mut out = Map {
            sources: self.sources.clone(),
            approach: [[0.0; WIDTH]; HEIGHT],
            flee_cowardly: [[0.0; WIDTH]; HEIGHT],
            flee_bravely: [[0.0; WIDTH]; HEIGHT],
        };
        out.clone_from(self);
        out
    }

    // Overwrite the provided array, rather than allocate a new one.
    fn clone_from(&mut self, source: &Map) {
        self.sources = source.sources.clone();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.approach[y][x] = source.approach[y][x];
                self.flee_cowardly[y][x] = source.flee_cowardly[y][x];
                self.flee_bravely[y][x] = source.flee_bravely[y][x];
            }
        }
    }
}

impl Debug for Map {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        try!(write!(formatter, "<Map "));
        try!(formatter.debug_list().entries(self.sources.iter()).finish());
        write!(formatter, ">")
    }
}

/// Create a new map from the given sources.
pub fn new_map_from_sources(sources: Vec<Point>, world: &World) -> Map {
    let mut out = Map {
        sources: sources.clone(),
        approach: [[f64::MAX; WIDTH]; HEIGHT],
        flee_cowardly: [[f64::MAX; WIDTH]; HEIGHT],
        flee_bravely: [[f64::MAX; WIDTH]; HEIGHT],
    };

    // Make the goals all global minima.
    for source in &sources {
        out.approach[source.y][source.x] = 0.0;
    }

    // Fill in the rest of the approach map.
    flood_fill(&mut out.approach, &sources, world);

    // Compute the fleeing maps and find their global minima.
    let mut minima: Vec<Point> = Vec::new();
    let mut minimal: f64 = f64::MAX;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if out.approach[y][x] != f64::MAX {
                out.flee_cowardly[y][x] = out.approach[y][x] * COWARDICE_COEFF;
                out.flee_bravely[y][x] = out.approach[y][x] * BRAVERY_COEFF;

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
fn flood_fill(map: &mut [[f64; WIDTH]; HEIGHT], minima: &Vec<Point>, world: &World) {
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
            let my_min = local_min + 1.0;
            if my_min < map[pos.y][pos.x] {
                map[pos.y][pos.x] = my_min;
                for a in adj {
                    queue.push_back(a);
                }
            }
        }
    }
}
