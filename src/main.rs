//! This is the entry point of the game.

// Turn on some code quality linting
#![warn(missing_copy_implementations, missing_debug_implementations, missing_docs, trivial_casts,
        trivial_numeric_casts, unused_extern_crates, unused_import_braces, unused_qualifications,
        unused_results)]

extern crate rand;
extern crate rogue_mayor;

use rand::Rng;
use rogue_mayor::dijkstra_map::*;
use rogue_mayor::grid::*;
use rogue_mayor::language::Language;
use rogue_mayor::mobiles::*;
use rogue_mayor::statics::*;
use rogue_mayor::templates::*;
use rogue_mayor::types::*;
use rogue_mayor::ui::*;
use rogue_mayor::ui::sdlui::*;
use std::collections::BTreeMap;
use std::env;

fn main() {
    if env::args().nth(1) == Some("namegen".to_string()) {
        namegen()
    } else if env::args().nth(1) == Some("chargen".to_string()) {
        chargen()
    } else {
        game()
    }
}

/// Roll a language and print some examples.
fn namegen() {
    let mut rng = rand::thread_rng();
    let mut lang = Language::new(&mut rng);

    for _ in 0..25 {
        let name = lang.gen_personal(&mut rng);
        println!("{}", name);
    }
}

/// Roll a character and print their stats.
fn chargen() {
    let mut rng = rand::thread_rng();
    let mut lang = Language::new(&mut rng);
    let (ty, mob) = match rng.gen_range(0, 3) {
        0 => ("Adventurer", Mobile::gen_adventurer(&mut rng, &mut lang)),
        1 => ("Child", Mobile::gen_child(&mut rng, &mut lang)),
        _ => ("Ordinary Boring Adult", Mobile::gen_adult(&mut rng, &mut lang)),
    };

    println!("{} the {} ({} years old)\n", mob.name, ty, mob.age);

    // Personality traits
    let mut f = false;
    if mob.is_avaricious {
        println!("Is particularly avaricious");
        f = true;
    }
    if mob.is_brave {
        println!("Is particularly brave");
        f = true;
    }
    if mob.is_envious {
        println!("Is particularly envious");
        f = true;
    }
    if mob.is_gluttonous {
        println!("Is particularly gluttonous");
        f = true;
    }
    if mob.is_slothful {
        println!("Is particularly slothful");
        f = true;
    }
    if f {
        println!("");
    }

    // History
    println!("Biography:");
    for &(age, event) in &mob.history {
        match event {
            LifeEvent::Born => println!("\tAge {}: Born.", age),
            LifeEvent::Raised { childhood } => {
                println!("\tAge {}: Raised with a {:?} childhood.", age, childhood)
            }
            LifeEvent::Learned { package } => {
                println!("\tAge {}: Gained experience in {:?}.", age, package)
            }
            LifeEvent::Onset => println!("\tAge {}: Became an adventurer.", age),
        }
    }

    // Stats
    println!("\nAttributes:");
    println!("\tagility: {}", mob.agility);
    println!("\tconstitution: {}", mob.constitution);
    println!("\tendurance: {}", mob.endurance);
    println!("\trecuperation: {}", mob.recuperation);
    println!("\tstrength: {}", mob.strength);
    println!("\ttoughness: {}", mob.toughness);
    println!("\tanimal: {}", mob.animal);
    println!("\tbargain: {}", mob.bargain);
    println!("\tcharm: {}", mob.charm);
    println!("\tchirurgy: {}", mob.chirurgy);
    println!("\tcraft: {}", mob.craft);
    println!("\tempathy: {}", mob.empathy);
    println!("\tforage: {}", mob.forage);
    println!("\tguile: {}", mob.guile);
    println!("\theal: {}", mob.heal);
    println!("\thunt: {}", mob.hunt);
    println!("\tintuition: {}", mob.intuition);
    println!("\tlore: {}", mob.lore);
    if mob.competence_bow > 0 {
        println!("\tcompetence (bow): {}", mob.competence_bow);
    }
    if mob.competence_great > 0 {
        println!("\tcompetence (great): {}", mob.competence_great);
    }
    if mob.competence_shield > 0 {
        println!("\tcompetence (shield): {}", mob.competence_shield);
    }
    if mob.competence_single > 0 {
        println!("\tcompetence (single): {}", mob.competence_single);
    }
    if mob.competence_spear > 0 {
        println!("\tcompetence (spear): {}", mob.competence_spear);
    }
    if mob.competence_staff > 0 {
        println!("\tcompetence (staff): {}", mob.competence_staff);
    }
    if mob.competence_sword > 0 {
        println!("\tcompetence (sword): {}", mob.competence_sword);
    }
    if mob.competence_warhammer > 0 {
        println!("\tcompetence (warhammer): {}", mob.competence_warhammer);
    }
    if mob.profession_adventurer > 0 {
        println!("\tprofession (adventurer): {}", mob.profession_adventurer);
    }
    if mob.profession_animalhandler > 0 {
        println!("\tprofession (animal handler): {}",
                 mob.profession_animalhandler);
    }
    if mob.profession_apothecarist > 0 {
        println!("\tprofession (apothecarist): {}",
                 mob.profession_apothecarist);
    }
    if mob.profession_appraiser > 0 {
        println!("\tprofession (appraiser): {}", mob.profession_appraiser);
    }
    if mob.profession_cutter > 0 {
        println!("\tprofession (cutter): {}", mob.profession_cutter);
    }
    if mob.profession_farmer > 0 {
        println!("\tprofession (farmer): {}", mob.profession_farmer);
    }
    if mob.profession_innkeeper > 0 {
        println!("\tprofession (innkeeper): {}", mob.profession_innkeeper);
    }
    if mob.profession_laborer > 0 {
        println!("\tprofession (laborer): {}", mob.profession_laborer);
    }
    if mob.profession_tinker > 0 {
        println!("\tprofession (tinker): {}", mob.profession_tinker);
    }
    if mob.profession_trader > 0 {
        println!("\tprofession (trader): {}", mob.profession_trader);
    }
    if mob.profession_woodsman > 0 {
        println!("\tprofession (woodsman): {}", mob.profession_woodsman);
    }
}

/// Play the game!
fn game() {
    println!("Welcome to Rogue Mayor!");

    match SdlUI::new() {
        Ok(mut ui) => {
            // Set up the state.
            let mut maps: Maps = Maps::new();
            let mut mobs: BTreeMap<Point, Mobile> = BTreeMap::new();
            let mut world: World = World::new();
            world.cursor = SdlUI::initial_cursor();

            // Everyone likes welcomes.
            world.log(Message {
                msg: "Welcome to Rogue Mayor!".to_string(),
                loc: None,
            });

            // Testing stuff
            let pos = Point { x: 25, y: 25 };
            world.statics.set(pos, Some(Static::new(StaticTag::Dungeon)));
            maps.mutget(MapTag::Adventure).add_source(pos, &world);
            let _ = world.sources.insert(pos, MapTag::Adventure);

            // Game loop
            'game: loop {
                // Update all mobs: clone the mob map, as we're going to be mutating it then, for
                // each mob in the original map, check if it's still in the old map (it might have
                // been killed) and step it. This may also mutate the maps, if a mob performs a
                // map-relevant action.
                let mut new_mobs = mobs.clone();
                for (pos, mob) in mobs.iter_mut() {
                    // This check is perhaps too lenient. For example, if Mob A destroys Mob B and
                    // creates Mob C in the same place, then Mob C will get a turn, even though it
                    // is new. This can be explained away by saying that Mob B wasn't destroyed,
                    // merely transformed into Mob C...
                    if new_mobs.contains_key(pos) {
                        mob.step(pos.clone(), &mut new_mobs, &mut maps, &mut world);
                    }
                }
                mobs = new_mobs;

                let mut has_advanced = true;
                'ui: loop {
                    // Render the world now, so the player has an up-to-date view before they are
                    // prompted for their next action.
                    ui.render(&mobs, &maps, &world, has_advanced);
                    has_advanced = false;

                    // Prompt for user input and perform the desired action.
                    let action = ui.input(world.cursor);
                    match action {
                        Command::BuildTemplate => {
                            world.build(&mut maps);
                            world.template = None;
                        }
                        Command::Quit => break 'game,
                        Command::Render => {}
                        Command::SetCursorTo(c) => world.cursor = c,
                        Command::SetTemplateTo(t) => world.template = Some(Template::new(t)),
                        Command::Step => break 'ui,
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
