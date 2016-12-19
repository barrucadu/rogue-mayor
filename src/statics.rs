//! Statics: things which have an unmoving presence in the world. Terrain, walls, doors, tables, etc
//! all fit into this category.

use dijkstra_map::*;

/// Things which have a fixed presence in the world.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Static {
    /// What this is.
    pub tag: StaticTag,
    /// Acts as an obstruction to mobiles and heatmap flow.
    pub is_impassable: bool,
    /// Acts as an obstruction to line-of-sight.
    pub is_opaque: bool,
}

/// Types of `Static`s.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StaticTag {
    /// The dungeon entrance: source of Adventure, impassable, not opaque.
    Dungeon,
    /// The counter of a general store: source of GeneralTrade, impassable, not opaque.
    GStoreCounter,
    /// The counter of an inn: source of Sustenance, impassable, not opaque.
    InnCounter,
    /// A wall: impassable, opaque.
    Wall,
    /// A bed: source of Rest, passable, not opaque.
    Bed,
    /// A door: passable, not opaque.
    Door,
}

impl Static {
    /// Construct a new `Static` from its tag.
    pub fn new(tag: StaticTag) -> Static {
        Static {
            tag: tag,
            is_impassable: match tag {
                StaticTag::Dungeon | StaticTag::GStoreCounter | StaticTag::InnCounter |
                StaticTag::Wall => true,
                _ => false,
            },
            is_opaque: match tag {
                StaticTag::Wall => true,
                _ => false,
            },
        }
    }

    /// The `MapTag` that this contributes to.
    pub fn maptag(&self) -> Option<MapTag> {
        match self.tag {
            StaticTag::Dungeon => Some(MapTag::Adventure),
            StaticTag::GStoreCounter => Some(MapTag::GeneralStore),
            StaticTag::InnCounter => Some(MapTag::Sustenance),
            StaticTag::Bed => Some(MapTag::Rest),
            StaticTag::Wall | StaticTag::Door => None,
        }
    }
}
