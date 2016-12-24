//! Mobiles: things which can move around the world. Citizens, animals, visitors, and monsters all
//! fall into this class.

pub mod ai;
pub mod gen;

use constants::*;
use dijkstra_map::*;
use grid::*;
use mobiles::ai::Task;
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
///   determines in part what it will do next.
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
    /// A task being actively worked towards.
    pub priority_task: Option<Task>,
    /// Things this mob cares about, and the relative weightings it assigns to each.
    pub desires: BTreeMap<MapTag, f64>,
    /// The location of the mob's home. This is where it returns when there is nothing else to do.
    pub home_pos: Point,

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
        // TODO: Possibly pick a new priority task.

        // TODO: Adjust desires.

        // Run the AI.
        self.ai(pos, mobs, maps, world);
    }
}
