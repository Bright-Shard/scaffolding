pub mod display;

use {display::Display, scaffolding::plugin_prelude::*};

#[derive(Default)]
pub struct ScaffoldingUiPlugin {}
impl Plugin for ScaffoldingUiPlugin {
    fn load(&mut self, world: &mut World) {
        world.add_state(Display::default());
    }
}
