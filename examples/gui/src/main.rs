use {scaffolding::prelude::*, scaffolding_ui::prelude::*};

fn main() {
    let mut world = World::default();
    world.add_plugin(ScaffoldingUiPlugin::default());
}
