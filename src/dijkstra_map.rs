//! Dijkstra maps.

///
/// See:
/// - http://www.roguebasin.com/index.php?title=The_Incredible_Power_of_Dijkstra_Maps
/// - http://www.roguebasin.com/index.php?title=Dijkstra_Maps_Visualized

use constants::*;
use grid::*;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::f64;
use std::fmt::{Debug, Error, Formatter};
use types::*;
use utils::*;

/// The collection of all Dijkstra maps.
pub struct Maps {
    /// Places offering adventure, such as the dungeon entrance.
    pub adventure: Map,
    /// Stores, every store sells every type of thing currently.
    pub general_store: Map,
    /// Places to rest, such as inns.
    pub rest: Map,
    /// Sources of food and drink, such as inns.
    pub sustenance: Map,
}

impl Debug for Maps {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        try!(write!(formatter, "Maps:"));
        try!(write!(formatter, "\n\tAdventure: "));
        try!(self.adventure.fmt(formatter));
        try!(write!(formatter, "\n\tGeneralStore: "));
        try!(self.general_store.fmt(formatter));
        try!(write!(formatter, "\n\tRest: "));
        try!(self.rest.fmt(formatter));
        try!(write!(formatter, "\n\tSustenance: "));
        self.sustenance.fmt(formatter)
    }
}

impl Maps {
    /// Construct empty maps.
    pub fn new() -> Maps {
        Maps {
            adventure: Map::new(),
            general_store: Map::new(),
            rest: Map::new(),
            sustenance: Map::new(),
        }
    }

    /// Look up a map by tag.
    pub fn get(&self, tag: MapTag) -> &Map {
        match tag {
            MapTag::Adventure => &self.adventure,
            MapTag::GeneralStore => &self.general_store,
            MapTag::Rest => &self.rest,
            MapTag::Sustenance => &self.sustenance,
        }
    }

    /// Look up a map by tag and return a mutable reference.
    pub fn mutget(&mut self, tag: MapTag) -> &mut Map {
        match tag {
            MapTag::Adventure => &mut self.adventure,
            MapTag::GeneralStore => &mut self.general_store,
            MapTag::Rest => &mut self.rest,
            MapTag::Sustenance => &mut self.sustenance,
        }
    }

    /// Recompute all maps. This should be called when a new obstruction is added.
    pub fn rebuild_all(&mut self, world: &World) {
        self.adventure.rebuild_from_sources(world);
        self.general_store.rebuild_from_sources(world);
        self.rest.rebuild_from_sources(world);
        self.sustenance.rebuild_from_sources(world);
    }
}

/// Symbolic names for the different maps.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MapTag {
    /// Places offering adventure, such as the dungeon entrance.
    Adventure,
    /// Stores, every store sells every type of thing currently.
    GeneralStore,
    /// Places to rest, such as inns.
    Rest,
    /// Sources of food and drink, such as inns.
    Sustenance,
}

/// A Dijkstra map, or heatmap.
pub struct Map {
    /// The sources (the global minima of the approach map).
    pub sources: Vec<Point>,
    /// Dijkstra map for approaching.
    pub approach: Grid<f64>,
    /// Dijkstra map for fleeing, where the fleeing creature in question is not willing to take many
    /// risks to escape.. This is the approaching map multipled by a negative coefficient and
    /// rescanned to smooth out corners and the like.
    pub flee_cowardly: Grid<f64>,
    /// Dijkstra map for fleeing, where the fleeing creature in question is willing to take more
    /// risks to escape. This is the approaching map multipled by a negative coefficient and
    /// rescanned to smooth out corners and the like.
    pub flee_bravely: Grid<f64>,
}

impl Debug for Map {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        try!(write!(formatter, "<Map "));
        try!(formatter.debug_list().entries(self.sources.iter()).finish());
        write!(formatter, ">")
    }
}

impl Map {
    /// A new empty map.
    fn new() -> Map {
        Map {
            sources: Vec::new(),
            approach: Grid::new(f64::MAX),
            flee_cowardly: Grid::new(f64::MAX),
            flee_bravely: Grid::new(f64::MAX),
        }
    }

    /// Add a new source to the map.
    pub fn add_source(&mut self, source: Point, world: &World) {
        self.add_source_no_rebuild(source);

        // Set the point to zero weighting and flood fill from that point.
        self.approach.set(source, 0.0);
        flood_fill(&mut self.approach, &vec![source], world);

        // Then completely recompute the flee maps.
        self.recompute_flee(world);
    }

    /// Add a new source to the map without rebuilding.
    pub fn add_source_no_rebuild(&mut self, source: Point) {
        self.sources.push(source);
    }

    /// Remove a source from the map.
    pub fn remove_source(&mut self, source: Point, world: &World) {
        self.remove_source_no_rebuild(source);
        self.rebuild_from_sources(world);
    }

    /// Remove source from the map without rebuilding.
    pub fn remove_source_no_rebuild(&mut self, source: Point) {
        self.sources.retain(|s| *s != source);
    }

    /// Recompute the map. This should be called when new obstructions are added.
    pub fn rebuild_from_sources(&mut self, world: &World) {
        // Reset the weights.
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let p = Point { x: x, y: y };
                self.approach.set(p, f64::MAX);
                self.flee_cowardly.set(p, f64::MAX);
                self.flee_bravely.set(p, f64::MAX);
            }
        }

        // Make the goals all global minima.
        for source in &self.sources {
            self.approach.set(*source, 0.0);
        }

        // Fill in the rest of the approach map.
        flood_fill(&mut self.approach, &self.sources, world);

        // Compute the fleeing maps and find their global minima.
        self.recompute_flee(world);
    }

    /// Recompute the fleeing maps. Not publically exported as it's called appropriately by other
    /// functions in here.
    fn recompute_flee(&mut self, world: &World) {
        let mut minima: Vec<Point> = Vec::new();
        let mut minimal: f64 = f64::MAX;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let p = Point { x: x, y: y };
                if self.approach.at(p) != f64::MAX {
                    self.flee_cowardly.set(p, self.approach.at(p) * COWARDICE_COEFF);
                    self.flee_bravely.set(p, self.approach.at(p) * BRAVERY_COEFF);

                    if self.flee_cowardly.at(p) == minimal {
                        minima.push(p);
                    } else if self.flee_cowardly.at(p) < minimal {
                        minima = vec![p];
                        minimal = self.flee_cowardly.at(p);
                    }
                }
            }
        }

        // Smooth the fleeing aps by flood filling from their minima.
        flood_fill(&mut self.flee_cowardly, &minima, world);
        flood_fill(&mut self.flee_bravely, &minima, world);
    }
}

/// Flood fill out from some points. When considering a new point, this behaves as follows:
/// - If the point is impassable, it keeps its current value.
/// - If the point is passable, it is assigned the value 1+cheapest neighbour.
///
/// This function assumes that the points given are the global minima, and may not perform properly
/// if that is not the case.
fn flood_fill(map: &mut Grid<f64>, minima: &Vec<Point>, world: &World) {
    let mut queue: VecDeque<Point> = VecDeque::with_capacity(WIDTH * HEIGHT / 2);
    for m in minima {
        queue.push_back(*m);
    }

    // Used later to check occupancy disregarding mobs.
    let empty = BTreeMap::new();

    // Flood fill.
    while let Some(pos) = queue.pop_front() {
        let val = map.at(pos);

        // Compute the local minima.
        let mut local_min = val;
        for dy in inclusive_range(-1, 1) {
            if (dy < 0 && pos.y == 0) || (dy > 0 && pos.y == HEIGHT - 1) {
                continue;
            }
            for dx in inclusive_range(-1, 1) {
                if (dx < 0 && pos.x == 0) || (dx > 0 && pos.x == WIDTH - 1) {
                    continue;
                }
                let here = map.at(Point {
                    x: signed_add(pos.x, dx),
                    y: signed_add(pos.y, dy),
                });
                if here < local_min {
                    local_min = here;
                }
            }
        }

        // If this results in a change of weight, push all the adjacent unoccupied tiles > the new
        // value+1. A special case is if the weight is 0, as that means we're flood filling from
        // this point outwards (and it naturally won't reduce!). A "better" check would be if `pos`
        // is in `minima`, but that is far more expensive than a simple floating point comparison.
        let my_min = local_min + 1.0;
        if my_min < val {
            map.set(pos, my_min);
        }
        if my_min < val || val == 0.0 {
            for dy in inclusive_range(-1, 1) {
                if (dy < 0 && pos.y == 0) || (dy > 0 && pos.y == HEIGHT - 1) {
                    continue;
                }
                for dx in inclusive_range(-1, 1) {
                    if (dx < 0 && pos.x == 0) || (dx > 0 && pos.x == WIDTH - 1) {
                        continue;
                    }
                    let p = Point {
                        x: signed_add(pos.x, dx),
                        y: signed_add(pos.y, dy),
                    };
                    if map.at(p) > my_min + 1.0 && !is_occupied(p, &empty, world) {
                        queue.push_back(p);
                    }
                }
            }
        }
    }
}
