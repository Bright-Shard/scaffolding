use crate::world::World;

pub trait Plugin: Default + 'static {
    fn load(&mut self, world: &mut World);
}
