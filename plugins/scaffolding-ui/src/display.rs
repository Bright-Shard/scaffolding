//! The module for [`Display`].

use scaffolding::{datatypes::ArenaVec, world::World};

pub mod gfx;

/// The [`Display`] is a bridge between ScaffoldingUI and lower-level APIs for
/// creating GUI apps. This type handles creating windows, getting user input,
/// drawing on windows, and any other OS quirks that need to be dealt with.
pub struct Display {
    /// All of the created windows.
    pub windows: ArenaVec<()>,
}
impl Display {
    pub fn new(_world: &mut World) -> Self {
        Self {
            windows: Default::default(),
        }
    }
}
