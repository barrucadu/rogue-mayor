//! All the types. This is just a placeholder module as things get implemented and spread out into
//! their own modules.

use dijkstra_map::*;
use grid::*;
use statics::*;
use std::collections::VecDeque;
use templates::*;

/// A command from the user.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Command {
    /// Build the active template at the world's cursor.
    BuildTemplate,
    /// Terminate.
    Quit,
    /// Re-render the UI.
    Render,
    /// Change the selected cell.
    SetCursorTo(Point),
    /// Change the active template.
    SetTemplateTo(Templates),
    /// Advance one turn.
    Step,
}

/// The state of the vsible map and the larger game world.
pub struct World {
    /// Things which have a fixed presence and location in the world.
    pub statics: Grid<Option<Static>>,
    /// Message log.
    pub messages: VecDeque<Message>,
    /// Selected cell.
    pub cursor: Point,
    /// Selected template.
    pub template: Option<Template>,
}

impl World {
    /// Construct a new world.
    pub fn new() -> World {
        World {
            statics: Grid::new(None),
            messages: VecDeque::new(),
            cursor: Point { x: 0, y: 0 },
            template: None,
        }
    }

    /// Log a new message.
    pub fn log(&mut self, msg: Message) {
        self.messages.push_front(msg);
    }

    /// Do a turn.
    pub fn step(&mut self) {}

    /// Build the active template at the cursor.
    pub fn build(&mut self, maps: &mut Maps) {
        if let Some(ref tpl) = self.template {
            for (p, &(s, t)) in &tpl.components {
                let pos = p.offset(self.cursor);
                self.statics.set(pos, Some(s));
                if let Some(tag) = t {
                    // Rebuild the maps at the end.
                    maps.mutget(tag).add_source_no_rebuild(pos);
                }
            }
            maps.rebuild_all(self);
        }
    }
}

/// A message consists of some text and an optional location. The UI intelligently handle the
/// location (eg, jump-to-location).
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Message {
    /// The message text.
    pub msg: String,
    /// The optional location.
    pub loc: Option<Point>,
}
