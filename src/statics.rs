//! Statics: things which have an unmoving presence in the world. Terrain, walls, doors, tables, etc
//! all fit into this category.

/// Things which have a fixed presence in the world, like walls.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Static {
    // impassable
    /// Solid walls
    Wall,
    // passable
    /// Doors
    Door,
}

impl Static {
    /// If true, this acts as an obstruction to mobiles and heatmap flow.
    pub fn impassable(&self) -> bool {
        match *self {
            Static::Wall => true,
            Static::Door => false,
        }
    }
}
