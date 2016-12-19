//! Mobiles: things which can move around the world. Citizens, animals, visitors, and monsters all
//! fall into this class.

use constants::*;
use dijkstra_map::*;
use grid::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::f64;
use types::*;
use utils::*;

/// Things which roam around in the world, like people and monsters.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Mobile {
    /// If the mob is especially brave or not. At the moment this only leads to greater risk-taking
    /// when fleeing something.
    pub is_brave: bool,

    /// Things this mob cares about, and the relative weightings it assigns to each.
    pub desires: BTreeMap<MapTag, f64>,
}

impl Mobile {
    /// Do a turn.
    pub fn step(&self,
                pos: Point,
                mobs: &mut BTreeMap<Point, Mobile>,
                maps: &mut Maps,
                world: &mut World) {
        // Compute a position to move to based on the desires of the mob.
        let new_pos = self.heatmap_ai(pos, maps, world);

        if new_pos == pos {
            // If we don't move, perform an action where we are and possibly adjust the desire
            // weights.
            let mut new_mob = self.clone();
            new_mob.interact_at_point(pos, pos, mobs, maps, world);
            let _ = mobs.insert(pos, new_mob);
        } else if is_occupied(new_pos, mobs, world) {
            // If the chosen point is occupied AND a local minimum, then it contains a (solid) goal
            // which we can interact with from this adjacent square. If not, we're just stuck for
            // this turn and sit on our hands (or claws, whatever).
            if self.heatmap_ai(new_pos, maps, world) == new_pos {
                let mut new_mob = self.clone();
                new_mob.interact_at_point(pos, new_pos, mobs, maps, world);
                let _ = mobs.insert(pos, new_mob);
            }
        } else {
            // Otherwise move to the new position.
            let _ = mobs.remove(&pos);
            let _ = mobs.insert(new_pos, self.clone());
        }
    }

    /// The heatmap AI: a very simple AI based on hill-climbing (or descending, rather). If there
    /// are multiple possible choices, pick the first considered.
    ///
    /// See:
    /// - http://www.roguebasin.com/index.php?title=The_Incredible_Power_of_Dijkstra_Maps
    /// - http://www.roguebasin.com/index.php?title=Dijkstra_Maps_Visualized
    fn heatmap_ai(&self, pos: Point, maps: &Maps, world: &World) -> Point {
        // Work out what sources are visible from here.
        let mut locally_visible = BTreeSet::new();
        for (p, tag) in &world.sources {
            if can_see(pos, *p, world) {
                locally_visible.insert(tag);
            }
        }

        // Find the minimum weighted sum of all the heatmaps in the local area:
        let mut new_pos = pos;
        let mut min_so_far = f64::MAX;
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

                // Compute the weight here.
                let mut weight_here = 0.0;
                for (tag, weight) in &self.desires {
                    let multiplier = if locally_visible.contains(tag) {
                        100.0
                    } else {
                        1.0
                    };
                    let wgt = *weight * multiplier;
                    let map = maps.get(*tag);
                    let delta = if wgt > 0.0 {
                            &map.approach
                        } else {
                            if self.is_brave {
                                &map.flee_bravely
                            } else {
                                &map.flee_cowardly
                            }
                        }
                        .at(Point { x: x, y: y });
                    weight_here += wgt * delta;
                }

                // And compare with the minimum seen so far.
                if weight_here < min_so_far {
                    new_pos = Point { x: x, y: y };
                    min_so_far = weight_here;
                }
            }
        }
        new_pos
    }

    /// Interact with a desired thing at a position reachable from the current one.
    fn interact_at_point(&mut self,
                         _: Point,
                         pos: Point,
                         _: &mut BTreeMap<Point, Mobile>,
                         _: &mut Maps,
                         world: &mut World) {
        if let Some(s) = world.statics.at(pos) {
            if let Some(tag) = s.maptag() {
                self.satisfy_desire(tag, 1.0);
            }
        }
    }

    /// Reduce the weight of a desire by a certain amount, to a minimum of 0.
    fn satisfy_desire(&mut self, tag: MapTag, by: f64) {
        if let Some(old) = self.desires.clone().get(&tag) {
            let new = old - by;
            let _ = self.desires.insert(tag, if new < 0.0 { 0.0 } else { new });
        }
    }
}
