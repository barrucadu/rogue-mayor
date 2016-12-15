//! The user interface.

pub mod sdlui;

use dijkstra_map::*;
use grid::*;
use mobiles::Mobile;
use std::collections::BTreeMap;
use types::*;

/// The UI. Implemented as a trait to allow for differing implementations.
pub trait UI {
    /// Render
    fn render(&mut self, &BTreeMap<Point, Mobile>, &Maps, &World);

    /// Await input.
    fn input(&mut self) -> Command;
}