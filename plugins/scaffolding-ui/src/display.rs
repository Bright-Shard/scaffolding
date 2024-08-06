//! The module for [`Display`].

use {
    platform::{Platform, PlatformTrait},
    scaffolding::{datatypes::ArenaVec, world::World},
};

pub mod gfx;
pub mod platform;

/// The [`Display`] is a bridge between ScaffoldingUI and lower-level APIs for
/// creating GUI apps. This type handles creating windows, getting user input,
/// drawing on windows, and any other OS quirks that need to be dealt with.
pub struct Display {
    /// All of the created windows.
    pub windows: ArenaVec<()>,
    platform: Platform,
}
impl Display {
    pub fn new(world: &mut World) -> Self {
        let platform = Platform::new(world).expect("Failed to connect to the OS' windowing server");

        Self {
            windows: Default::default(),
            platform,
        }
    }
}
