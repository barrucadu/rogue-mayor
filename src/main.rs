//! This is the entry point of the game.

// Turn on some code quality linting
#![warn(missing_copy_implementations, missing_debug_implementations, missing_docs, trivial_casts,
        trivial_numeric_casts, unused_extern_crates, unused_import_braces, unused_qualifications,
        unused_results)]

extern crate rogue_mayor;

use rogue_mayor::dijkstra_map::*;
use rogue_mayor::mobiles::*;
use rogue_mayor::types::*;
use rogue_mayor::ui::{UI, sdlui};
use std::collections::BTreeMap;

fn main() {
    println!("Welcome to Rogue Mayor!");

    match sdlui::new() {
        Ok(mut ui) => {
            // Set up the state.
            let mut maps: Maps = new_maps();
            let mut mobs: BTreeMap<Point, Mobile> = BTreeMap::new();
            let mut world: World = World {};

            // Game loop
            'game: loop {
                // Update all mobs: clone the mob map, as we're going to be mutating it then, for
                // each mob in the original map, check if it's still in the old map (it might have
                // been killed) and step it. This may also mutate the maps, if a mob performs a
                // map-relevant action.
                let mut new_mobs = mobs.clone();
                for (pos, mob) in &mobs {
                    // This check is perhaps too lenient. For example, if Mob A destroys Mob B and
                    // creates Mob C in the same place, then Mob C will get a turn, even though it
                    // is new. This can be explained away by saying that Mob B wasn't destroyed,
                    // merely transformed into Mob C...
                    if new_mobs.contains_key(pos) {
                        mob.step(pos.clone(), &mut new_mobs, &mut maps, &mut world);
                    }
                }
                mobs = new_mobs;

                let mut action = Command::Render;
                'ui: while action == Command::Render {
                    // Render the world now, so the player has an up-to-date view before they are
                    // prompted for their next action.
                    ui.render(&mobs, &maps, &world);

                    // Prompt for user input and perform the desired action.
                    action = ui.input();
                    match action {
                        Command::Skip | Command::Render => {}
                        Command::Quit => break 'game,
                    }
                }

                // Step the world state.
                world.step();
            }
        }
        Err(e) => panic!("Could not initialise SDL2: {}", e),
    }
}
