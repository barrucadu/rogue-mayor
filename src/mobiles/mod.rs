//! Mobiles: things which can move around the world. Citizens, animals, visitors, and monsters all
//! fall into this class.

pub mod gen;

use constants::*;
use dijkstra_map::*;
use grid::*;
use mobiles::gen::{Childhood, TrainingPackage};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::f64;
use types::*;
use utils::*;

/// Things which roam around in the world, like people and monsters.
///
/// Mobs have biography, desires, personality traits, and attributes.
///
/// - Biography has no effect on the game and is just flavour. The age and onset age affect
///   character generation, but not how the character then behaves once generated. Age might become
///   important later, once I implement the passage of in-world time.
///
/// - Desires are what the mob wants to do *right now*. The relative weight of the desires
///   determines what it will do next.
///
/// - Personality traits effect what the mob does. These can change over time after significant
///   personal events.
///
/// - Attributes affect the ability at tasks. These are in four classes:
///     - Physical: these are mostly important for adventurers going into the dungeon, until I
///       implement some sort of military/combat system.
///     - Mental: these are important for all mobs, and are derived from the sort of things a
///       non-adventurer might get up to.
///     - Competences: competence at using specific types of equipment. Only used by adventurers
///       currently.
///     - Profession: general competence in a specific job. They modify the relevant attributes
///       when performing a task as a part of the mob's job. For example, if someone need an item
///       identifying, a professional appraiser will do better than an innkeeper who just happens
///       to be very lore-wise. Each has a related building where the mob can ply their trade.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Mobile {
    // Biography
    /// The name. This doesn't affect anything, and is just flavour.
    pub name: String,
    /// How old the mob is.
    pub age: usize,
    /// Age at which the mob became an adventurer, if they are one.
    pub onset_age: Option<usize>,
    /// The developmental history of this mob. The `usize` is the age at which this happened. This
    /// is sorted ascending by year.
    pub history: Vec<(usize, LifeEvent)>,

    // Desires
    /// Things this mob cares about, and the relative weightings it assigns to each.
    pub desires: BTreeMap<MapTag, f64>,

    // Personality traits
    /// Increases the value the mob ascribes to items it is trying to sell.
    pub is_avaricious: bool,
    /// Leads to greater risk-taking when fleeing something.
    pub is_brave: bool,
    /// Increases the value the mob ascribes to items it is trying to buy.
    pub is_envious: bool,
    /// Increases the minimum food desire weighting.
    pub is_gluttonous: bool,
    /// Increases the minimum rest desire weighting.
    pub is_slothful: bool,

    // Attributes (physical)
    /// Ability to dodge attacks and avoid traps.
    pub agility: usize,
    /// Ability to recover from poison and disease.
    pub constitution: usize,
    /// Ability to do more work without needing rest.
    pub endurance: usize,
    /// Ability to recover while resting.
    pub recuperation: usize,
    /// Ability to wield weapons and armour.
    pub strength: usize,
    /// Ability to absorb damage.
    pub toughness: usize,

    // Attributes (mental)
    /// Productivity and longevity of animals under the mob's care.
    pub animal: usize,
    /// Ability to strike a favourable deal.
    pub bargain: usize,
    /// Ability to leave favourable impressions and make friends.
    pub charm: usize,
    /// Ability to heal wounds.
    pub chirurgy: usize,
    /// Ability to repair items.
    pub craft: usize,
    /// Ability understand others feelings.
    pub empathy: usize,
    /// Ability to find useful items in the wilderness.
    pub forage: usize,
    /// Ability to deceive others.
    pub guile: usize,
    /// Ability to identify healing ingredients and treat disease/poison.
    pub heal: usize,
    /// Ability to track and trap animals.
    pub hunt: usize,
    /// Ability to predict events.
    pub intuition: usize,
    /// Ability to accurately identify and value dungeon spoils.
    pub lore: usize,

    // Attributes (competencies)
    /// Bonus when using bows.
    pub competence_bow: usize,
    /// Bonus when using great weapons.
    pub competence_great: usize,
    /// Bonus when using shields.
    pub competence_shield: usize,
    /// Competence when using single weapons.
    pub competence_single: usize,
    /// Bonus when spears.
    pub competence_spear: usize,
    /// Bonus when using staves
    pub competence_staff: usize,
    /// Bonus when using swords.
    pub competence_sword: usize,
    /// Bonus when using warhammers.
    pub competence_warhammer: usize,

    // Attributes (profession)
    /// Bonus when acting as a professional adventurer.
    pub profession_adventurer: usize,
    /// Bonus when acting as a professional animal handler.
    pub profession_animalhandler: usize,
    /// Bonus when acting as a professional apothecarist.
    pub profession_apothecarist: usize,
    /// Bonus when acting as a professional appraiser.
    pub profession_appraiser: usize,
    /// Bonus when acting as a professional cutter.
    pub profession_cutter: usize,
    /// Bonus when acting as a professional farmer.
    pub profession_farmer: usize,
    /// Bonus when acting as a professional innkeeper.
    pub profession_innkeeper: usize,
    /// Bonus when acting as a professional laborer.
    pub profession_laborer: usize,
    /// Bonus when acting as a professional tinker.
    pub profession_tinker: usize,
    /// Bonus when acting as a professional trader.
    pub profession_trader: usize,
    /// Bonus when acting as a professional woodsman.
    pub profession_woodsman: usize,
}

/// Life events.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LifeEvent {
    Born,
    Raised { childhood: Childhood },
    Learned { package: TrainingPackage },
    Onset,
}

impl Mobile {
    /// Do a turn.
    pub fn step(&mut self,
                pos: Point,
                mobs: &mut BTreeMap<Point, Mobile>,
                maps: &mut Maps,
                world: &mut World) {
        // Compute a position to move to based on the desires of the mob.
        let new_pos = self.ai(pos, maps, world);

        if new_pos == pos {
            // If we don't move, perform an action where we are and possibly adjust the desire
            // weights.
            self.interact_at_point(pos, pos, mobs, maps, world);
        } else if is_occupied(new_pos, mobs, world) {
            // If the chosen point is occupied AND a local minimum, then it contains a (solid) goal
            // which we can interact with from this adjacent square. If not, we're just stuck for
            // this turn and sit on our hands (or claws, whatever).
            if self.ai(new_pos, maps, world) == new_pos {
                self.interact_at_point(pos, new_pos, mobs, maps, world);
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
    fn ai(&self, pos: Point, maps: &Maps, world: &World) -> Point {
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
