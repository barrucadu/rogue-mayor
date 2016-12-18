//! Construction templates.

use dijkstra_map::*;
use grid::*;
use statics::*;
use std::collections::BTreeMap;

// `from_grid` helper.
macro_rules! s{
    ($s:ident) => (Some(Static::$s))
}

/// The available templates
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Templates {
    /// An small inn with a bedroom.
    Inn,
    /// A small general store.
    GeneralStore,
}

/// A template is a list of statics objects to place, with (0,0) being the top-left corner of the
/// template.
#[derive(Clone, Debug)]
pub struct Template {
    /// The components of the template, including possible maptags to place.
    pub components: BTreeMap<Point, (Static, Option<MapTag>)>,
}

impl Template {
    /// Get a template.
    pub fn new(tpl: Templates) -> Template {
        match tpl {
            Templates::Inn => Template::inn(),
            Templates::GeneralStore => Template::general_store(),
        }
    }

    /// Rotate 90 degrees clockwise.
    pub fn clockwise(&mut self) {
        let mut components = BTreeMap::new();

        // Find the maximum y value.
        let mut off = 0;
        for point in self.components.keys() {
            if point.y > off {
                off = point.y;
            }
        }

        // Transform every point (x,y) to (off-y,x).
        for (point, s) in self.components.clone() {
            let p = Point {
                x: off - point.y,
                y: point.x,
            };
            let _ = components.insert(p, s);
        }

        self.components = components;
    }

    /// Rotate 90 degrees anticlockwise.
    pub fn anticlockwise(&mut self) {
        let mut components = BTreeMap::new();

        // Find the maximum x value.
        let mut off = 0;
        for point in self.components.keys() {
            if point.x > off {
                off = point.x;
            }
        }

        // Transform every point (x,y) to (y,off-x).
        for (point, s) in self.components.clone() {
            let p = Point {
                x: point.y,
                y: off - point.x,
            };
            let _ = components.insert(p, s);
        }

        self.components = components;
    }

    /// A general store.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn general_store() -> Template {
        /*    #####
         *    #   #
         *    #║#
         *    #   #
         *    ##║##
         */
        from_grid(&[vec![s!(Wall),     s!(Wall),       s!(Wall),      s!(Wall),s!(Wall)],
                    vec![s!(Wall),       None,           None,          None,  s!(Wall)],
                    vec![s!(Wall),s!(GStoreCounter),s!(GStoreCounter),s!(Door),s!(Wall)],
                    vec![s!(Wall),       None,           None,          None,  s!(Wall)],
                    vec![s!(Wall),     s!(Wall),       s!(Door),      s!(Wall),s!(Wall)]])
    }


    /// An inn.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn inn() -> Template {
        /*    ##############
         *    #Θ  Θ  Θ#    #
         *    #       ║   #
         *    #########   #
         *            ║   #
         *            ######
         */
        from_grid(&[vec![s!(Wall),s!(Wall),s!(Wall),s!(Wall),s!(Wall),s!(Wall),s!(Wall),s!(Wall),s!(Wall),s!(Wall),s!(Wall),   s!(Wall),   s!(Wall),s!(Wall)],
                    vec![s!(Wall),s!(Bed),    None, s!(Bed),   None,  s!(Bed), s!(Wall),  None,    None,    None,    None,        None,      None,  s!(Wall)],
                    vec![s!(Wall),  None,     None,   None,    None,    None,  s!(Door),  None,    None,    None,    None,  s!(InnCounter),  None,  s!(Wall)],
                    vec![s!(Wall),s!(Wall),s!(Wall),s!(Wall),s!(Wall),s!(Wall),s!(Wall),  None,    None,    None,    None,  s!(InnCounter),  None,  s!(Wall)],
                    vec![  None,    None,    None,    None,    None,    None,  s!(Door),  None,    None,    None,    None,  s!(InnCounter),  None,  s!(Wall)],
                    vec![  None,    None,    None,    None,    None,    None,  s!(Wall),s!(Wall),s!(Wall),s!(Wall),s!(Wall),   s!(Wall),   s!(Wall),s!(Wall)]])
    }
}

/// Construct components from a grid. TODO: make this a macro.
fn from_grid(grid: &[Vec<Option<Static>>]) -> Template {
    let mut components = BTreeMap::new();

    for y in 0..grid.len() {
        for x in 0..grid[y].len() {
            if let Some(s) = grid[y][x] {
                let _ = components.insert(Point { x: x, y: y }, (s, s.maptag()));
            }
        }
    }

    Template { components: components }
}
