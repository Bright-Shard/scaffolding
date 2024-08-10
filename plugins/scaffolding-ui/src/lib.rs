pub mod prelude {
    pub use crate::ScaffoldingUiPlugin;
}

pub mod display;

use {display::Display, scaffolding::plugin_prelude::*};

#[derive(Default)]
pub struct ScaffoldingUiPlugin {}
impl Plugin for ScaffoldingUiPlugin {
    fn load(&mut self, world: &mut World) {
        let display = Display::new(world);
        world.add_singleton(display);
    }
}
