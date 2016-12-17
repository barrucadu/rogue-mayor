//! Statics: things which have an unmoving presence in the world. Terrain, walls, doors, tables, etc
//! all fit into this category.

use dijkstra_map::*;

/// Things which have a fixed presence in the world, like walls.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Static {
    // impassable
    /// The dungeon entrance: provides adventure.
    Dungeon,
    /// The counter of a general store: provides trade.
    GStoreCounter,
    /// The counter of an inn: provides sustenance.
    InnCounter,
    /// Solid walls
    Wall,
    // passable
    /// A bed: provides rest.
    Bed,
    /// Doors
    Door,
}

impl Static {
    /// If true, this acts as an obstruction to mobiles and heatmap flow.
    pub fn impassable(&self) -> bool {
        match *self {
            Static::Dungeon | Static::GStoreCounter | Static::InnCounter | Static::Wall => true,
            Static::Bed | Static::Door => false,
        }
    }

    /// The `MapTag` that this contributes to.
    pub fn maptag(&self) -> Option<MapTag> {
        match *self {
            Static::Dungeon => Some(MapTag::Adventure),
            Static::GStoreCounter => Some(MapTag::GeneralStore),
            Static::InnCounter => Some(MapTag::Sustenance),
            Static::Bed => Some(MapTag::Rest),
            Static::Wall | Static::Door => None,
        }
    }
}
