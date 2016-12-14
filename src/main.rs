//! This is the entry point of the game.

// Turn on some code quality linting
#![warn(missing_copy_implementations, missing_debug_implementations, missing_docs, trivial_casts,
        trivial_numeric_casts, unused_extern_crates, unused_import_braces, unused_qualifications,
        unused_results)]

use std::collections::BTreeMap;
use std::fmt::{Debug, Error, Formatter};

/// The width of the playable map.
const WIDTH: usize = 500;

/// The height of the playable map.
const HEIGHT: usize = 300;

/// A type of heatmap: each maptag has an associated map.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MapTag {
}

/// A location in 2d space.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Point {
    x: usize,
    y: usize,
}

/// Things which roam around in the world, like people and monsters.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Mobile {
}

impl Mobile {
    /// Do a turn.
    pub fn step(&self,
                _: Point,
                _: &mut BTreeMap<Point, Mobile>,
                _: &mut BTreeMap<MapTag, Map>,
                _: &mut World) {
    }
}

/// A heatmap.
#[derive(Copy)]
pub struct Map([[usize; WIDTH]; HEIGHT]);

impl Clone for Map {
    fn clone(&self) -> Map {
        *self
    }

    // Overwrite the provided array, rather than allocate a new one.
    fn clone_from(&mut self, source: &Map) {
        let Map(me) = *source;
        let Map(mut out) = *self;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                out[y][x] = me[y][x];
            }
        }
    }
}

impl Debug for Map {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let Map(me) = *self;
        let mut has_prior = false;

        try!(write!(formatter, "["));
        for row in me.iter() {
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
}

/// A command from the user.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Command {
    /// Advance one turn without doing any user action.
    Skip,
    /// Terminate.
    Quit,
}

/// Wait for a command from the user.
pub fn input() -> Command {
    Command::Quit
}

/// The state of the vsible map and the larger game world.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct World {
}

impl World {
    /// Do a turn.
    pub fn step(&mut self) {}
}

/// The UI. Implemented as a trait to allow for differing implementations.
pub trait UI {
    /// Render
    fn render(&mut self, &BTreeMap<Point, Mobile>, &BTreeMap<MapTag, Map>, &World);
}

/// A basic user interface.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BasicUI {}

impl UI for BasicUI {
    fn render(&mut self, _: &World, _: &BTreeMap<Point, Mobile>, _: &BTreeMap<MapTag, Map>) {}
}

fn main() {
    println!("Welcome to Rogue Mayor!");

    let mut maps: BTreeMap<MapTag, Map> = BTreeMap::new();
    let mut mobs: BTreeMap<Point, Mobile> = BTreeMap::new();
    let mut world: World = World {};
    let mut ui: BasicUI = BasicUI {};

    // Game loop
    'game: loop {
        // Update all mobs: clone the mob map, as we're going to be mutating it then, for each mob
        // in the original map, check if it's still in the old map (it might have been killed) and
        // step it. This may also mutate the maps, if a mob performs a map-relevant action.
        let mut new_mobs = mobs.clone();
        for (pos, mob) in &mobs {
            // This check is perhaps too lenient. For example, if Mob A destroys Mob B and creates
            // Mob C in the same place, then Mob C will get a turn, even though it is new. This can
            // be explained away by saying that Mob B wasn't destroyed, merely transformed into Mob
            // C...
            if new_mobs.contains_key(pos) {
                mob.step(pos.clone(), &mut new_mobs, &mut maps, &mut world);
            }
        }
        mobs = new_mobs;

        // Prompt for user input.
        let action = input();

        // Perform the user action.
        match action {
            Command::Skip => {}
            Command::Quit => break 'game,
        }

        // Step the world state.
        world.step();

        // Finally, render.
        ui.render(&world, &mobs, &maps);
    }
}
