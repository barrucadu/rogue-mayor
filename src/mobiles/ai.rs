//! Mobile AI.

use constants::{HEIGHT, WIDTH};
use dijkstra_map::{Map, MapTag, Maps};
use grid::Point;
use mobiles::Mobile;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::f64;
use types::World;
use utils::*;

/// Multiplier to the weight of a desire if a source can be seen.
const SIGHT_BONUS: f64 = 100.0;

impl Mobile {
    /// The AI. This uses a behaviour tree-style approach to decision making, see the in-code
    /// comments.
    pub fn ai(&mut self,
              pos: Point,
              mobs: &mut BTreeMap<Point, Mobile>,
              maps: &mut Maps,
              world: &mut World) {
        // 1. INTERACT AT POINT
        //
        // Can the mob interact with something that it wants to interact with?
        if let Some(target_pos) = self.ai_interact_nearby(pos, maps) {
            if self.ai_interact_at_point_commit(world, target_pos) {
                return;
            }
        }

        // 2. PRIORITY TASK
        //
        // Does the mob have something that it absolutely must do, as a matter of utmost priority?
        // Furthermore, can it make useful progress towards completing that task? If so, do that!
        if let Some(ref task) = self.priority_task {
            if let Some(new_pos) = self.ai_advance_task(pos, world, task) {
                if self.ai_move_commit(pos, mobs, world, new_pos) {
                    return;
                }
            }
        }

        // 3. HEATMAP TRAVEL
        //
        // Take the desire-weighted sum of the heatmaps, and determine if there's a place we'd like
        // to move towards.
        //
        // To encourage a "sensible" pathing behaviour, check if there are any desires we can see
        // the source of from this point. If so, we *particularly* want to move towards it (or away
        // from it, if it's scary), even if there are other more pressing desires. This discourages
        // back-and-forth travel.
        if let Some(new_pos) = self.ai_heatmap_wsum(pos, maps, world) {
            if self.ai_move_commit(pos, mobs, world, new_pos) {
                return;
            }
        }

        // 4. WANDERING
        //
        // The mob has reached a state of zen. It has nothing left to do in this mortal
        // plane. Satori is within reach. Just wander home.
        if let Some(new_pos) = self.ai_pathfind(pos, world, self.home_pos) {
            if self.ai_move_commit(pos, mobs, world, new_pos) {
                return;
            }
        }

        // 5. GIVING UP
        //
        // Just stay where we are.
    }

    // The `ai_*` functions do not modify the state of the game, and return an `Option` indicating
    // success (with further information) or failure.

    /// Determine if there is a nearby point we can interact with to satisfy a desire.
    fn ai_interact_nearby(&self, pos: Point, maps: &Maps) -> Option<Point> {
        // At the moment, just see if any of the >0.0-weighted heatmaps have an adjacent source.
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

                // Check the weights here.
                for (tag, weight) in &self.desires {
                    let new_pos = Point { x: x, y: y };
                    if *weight > 0.0 && maps.get(*tag).approach.at(new_pos) == 0.0 {
                        return Some(new_pos);
                    }
                }
            }
        }

        None
    }

    /// Determine a position to move to which will advance the given task.
    fn ai_advance_task(&self, pos: Point, world: &World, target: &Task) -> Option<Point> {
        match target {
            &Task::MoveTo(target_point) => self.ai_pathfind(pos, world, target_point),
        }
    }

    /// Choose a point to move to by taking the desire-weighted sum of the heatmaps. If a weight is
    /// negative, flee. If the source is visible, intensify the weight.
    ///
    /// See:
    /// - http://www.roguebasin.com/index.php?title=The_Incredible_Power_of_Dijkstra_Maps
    /// - http://www.roguebasin.com/index.php?title=Dijkstra_Maps_Visualized
    fn ai_heatmap_wsum(&self, pos: Point, maps: &Maps, world: &World) -> Option<Point> {
        // Work out what sources are visible from here.
        let mut locally_visible = BTreeSet::new();
        for (p, tag) in &world.sources {
            if can_see(pos, *p, world) {
                locally_visible.insert(tag);
            }
        }

        // Find the minimum weighted sum of all the heatmaps in the local area:
        let mut new_pos = None;
        let mut min_so_far = f64::MAX;
        for dy in inclusive_range(-1, 1) {
            if (dy < 0 && pos.y == 0) || (dy > 0 && pos.y == HEIGHT - 1) {
                continue;
            }
            let y = signed_add(pos.y, dy);
            for dx in inclusive_range(-1, 1) {
                if (dx < 0 && pos.x == 0) || (dx > 0 && pos.x == WIDTH - 1) ||
                   (dy == 0 && dx == 0) {
                    continue;
                }
                let x = signed_add(pos.x, dx);

                // Compute the weight here.
                let mut weight_here = 0.0;
                for (tag, weight) in &self.desires {
                    let multiplier = if locally_visible.contains(tag) {
                        SIGHT_BONUS
                    } else {
                        1.0
                    };
                    let wgt = *weight * multiplier;
                    let map = maps.get(*tag);
                    let delta = if wgt >= 0.0 {
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
                if weight_here < min_so_far && weight_here != 0.0 {
                    new_pos = Some(Point { x: x, y: y });
                    min_so_far = weight_here;
                }
            }
        }

        // Return the position, which is `None` if no candidate had a nonzero weight.
        new_pos
    }

    /// Find a path to the target and return the first point. If this returns `None` then the point
    /// is inaccessible!
    fn ai_pathfind(&self, pos: Point, world: &World, target: Point) -> Option<Point> {
        // For laziness, just re-use the existing dijkstra map machinery. This does a huge amount of
        // wasted computation!
        Map::new(vec![target], world).get_new_pos(target)
    }

    // The `ai_*_commit` functions actually modify the state of the game, and return a simple
    // indicator of success.

    /// Move to a new position if possible. Returns `false` if the move cannot go ahead.
    fn ai_move_commit(&self,
                      pos: Point,
                      mobs: &mut BTreeMap<Point, Mobile>,
                      world: &World,
                      new_pos: Point)
                      -> bool {
        if is_occupied(new_pos, mobs, world) {
            false
        } else {
            let _ = mobs.remove(&pos);
            let _ = mobs.insert(new_pos, self.clone());
            true
        }
    }

    /// Interact with the given target.
    fn ai_interact_at_point_commit(&mut self, world: &mut World, target_pos: Point) -> bool {
        // For now, the only interaction we have is satisfying a desire.
        if let Some(s) = world.statics.at(target_pos) {
            if let Some(tag) = s.maptag() {
                if let Some(old) = self.desires.clone().get(&tag) {
                    let new = old - 1.0;
                    let _ = self.desires.insert(tag, if new < 0.0 { 0.0 } else { new });
                    return true;
                }
            }
        }

        false
    }
}

/// Specific tasks that an AI can perform.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Task {
    /// Move to the given location.
    MoveTo(Point),
}
