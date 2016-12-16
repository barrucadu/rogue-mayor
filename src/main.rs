//! This is the entry point of the game.

// Turn on some code quality linting
#![warn(missing_copy_implementations, missing_debug_implementations, missing_docs, trivial_casts,
        trivial_numeric_casts, unused_extern_crates, unused_import_braces, unused_qualifications,
        unused_results)]

extern crate rand;
extern crate rogue_mayor;

use rand::distributions::{IndependentSample, Range};
use rogue_mayor::dijkstra_map::*;
use rogue_mayor::grid::*;
use rogue_mayor::mobiles::*;
use rogue_mayor::statics::*;
use rogue_mayor::types::*;
use rogue_mayor::ui::*;
use rogue_mayor::ui::sdlui::*;
use std::collections::BTreeMap;

fn main() {
    println!("Welcome to Rogue Mayor!");

    match SdlUI::new() {
        Ok(mut ui) => {
            // Set up the state.
            let mut maps: Maps = Maps::new();
            let mut mobs: BTreeMap<Point, Mobile> = BTreeMap::new();
            let mut world: World = World::new();

            // Everyone likes welcomes.
            world.log(Message {
                msg: "Welcome to Rogue Mayor!".to_string(),
                loc: None,
            });

            // Testing stuff
            inn(Point { x: 2, y: 2 }, &mut maps, &mut world);
            shop(Point { x: 4, y: 7 },
                 Static::GStoreCounter,
                 MapTag::GeneralStore,
                 &mut maps,
                 &mut world);
            dungeon(Point { x: 25, y: 25 }, &mut maps, &mut world);
            maps.rebuild_all(&world);

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

                    // Testing the message log.
                    world.log(Message {
                        msg: format!("You chose {:?}", action).to_string(),
                        loc: None,
                    });
                }

                // Step the world state.
                world.step();
            }
        }
        Err(e) => panic!("Could not initialise SDL2: {}", e),
    }
}

// Helper functions
fn inn(at: Point, maps: &mut Maps, world: &mut World) {
    // Bedroom
    room(at, 15, 3, world);

    // Beds
    for i in 0..7 {
        let p = Point {
            x: at.x + 1 + 2 * i,
            y: at.y + 1,
        };
        world.statics.set(p, Some(Static::Bed));
        maps.mutget(MapTag::Rest).add_source(Point { x: p.x, y: p.y }, &world);
    }

    // Inn proper
    shop(Point {
             x: at.x + 15,
             y: at.y,
         },
         Static::InnCounter,
         MapTag::Sustenance,
         maps,
         world);

    // Door between bedroom and inn
    world.statics.set(Point {
                          x: at.x + 15,
                          y: at.y + 2,
                      },
                      Some(Static::Door));
}

fn shop(at: Point, counter: Static, tag: MapTag, maps: &mut Maps, world: &mut World) {
    let mut rng = rand::thread_rng();

    // Walls
    let side = Range::new(5, 7);
    let width = side.ind_sample(&mut rng);
    let height = side.ind_sample(&mut rng);
    room(at, width, height, world);

    // Counter
    let mut pos = Point {
        x: at.x + width / 2,
        y: at.y + height / 2,
    };
    world.statics.set(pos, Some(counter));
    maps.mutget(tag).add_source(pos, &world);

    // Door
    match Range::new(0, 4).ind_sample(&mut rng) {
        0 => pos.x = at.x,
        1 => pos.x = at.x + width,
        2 => pos.y = at.y,
        _ => pos.y = at.y + height,
    }
    world.statics.set(pos, Some(Static::Door));
}

fn room(at: Point, width: usize, height: usize, world: &mut World) {
    for dy in 0..height {
        let mut p = Point {
            x: at.x,
            y: at.y + dy,
        };
        world.statics.set(p, Some(Static::Wall));
        p.x += width;
        world.statics.set(p, Some(Static::Wall));
    }

    for dx in 0..width {
        let mut p = Point {
            x: at.x + dx,
            y: at.y,
        };
        world.statics.set(p, Some(Static::Wall));
        p.y += height;
        world.statics.set(p, Some(Static::Wall));
    }

    world.statics.set(Point {
                          x: at.x + width,
                          y: at.y + height,
                      },
                      Some(Static::Wall));
}

fn dungeon(at: Point, maps: &mut Maps, world: &mut World) {
    world.statics.set(at, Some(Static::Dungeon));
    maps.mutget(MapTag::Adventure).add_source(at, &world);
}
