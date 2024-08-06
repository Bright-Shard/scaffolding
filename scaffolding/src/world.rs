//! Defines the [`World`], and types that interact with it.

pub mod executable;
pub mod executable_args;
pub mod mutation;
pub mod plugin;

use core::ops::{Deref, DerefMut};

pub use executable::*;
pub use executable_args::*;
pub use mutation::*;
pub use plugin::*;

use {
    crate::datatypes::{ArenaVec, TypeMap},
    alloc::boxed::Box,
    core::any::Any,
};

/// A version of the [`World`] that is guaranteed to be immutable.
///
/// This exists because the [`World`] can do things like queue mutations from
/// just an `&self` reference. This type, on the other hand, cannot, which makes
/// it safer in multithreaded contexts.
pub struct ImmutableWorld {
    pub plugins: TypeMap,
    pub states: TypeMap,
    mutations: ArenaVec<Box<dyn UnsizedMutation>>,
}
impl ImmutableWorld {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacities(
        num_states: usize,
        num_plugins: usize,
        states_memory: usize,
        plugins_memory: usize,
    ) -> Self {
        Self {
            plugins: TypeMap::new(num_plugins, plugins_memory),
            states: TypeMap::new(num_states, states_memory),
            mutations: ArenaVec::default(),
        }
    }

    #[inline(always)]
    pub fn try_get_state<S: Any>(&self) -> Option<&S> {
        self.states.get()
    }
    #[inline(always)]
    pub fn get_state<S: Any>(&self) -> &S {
        self.states.get().unwrap_or_else(||
                panic!(
                    "Scaffolding error: Tried to load state of type `{}`, but it wasn't put in the world. Did you forget to load a plugin?",
                    core::any::type_name::<S>()
                )
            )
    }

    pub fn has_loaded_plugin<P: Plugin>(&self) -> bool {
        self.plugins.contains::<P>()
    }
}
impl Default for ImmutableWorld {
    fn default() -> Self {
        Self::with_capacities(100, 10, 1_000, 100)
    }
}

pub struct World {
    immut: ImmutableWorld,
}
impl World {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn with_capacities(
        num_states: usize,
        num_plugins: usize,
        states_memory: usize,
        plugins_memory: usize,
    ) -> Self {
        Self {
            immut: ImmutableWorld::with_capacities(
                num_states,
                num_plugins,
                states_memory,
                plugins_memory,
            ),
        }
    }

    /// Run an [`Executable`] with the data in this [`World`].
    pub fn execute<E: IntoExecutable<Args, Output = Output>, Args, Output>(
        &mut self,
        executable: E,
        env: Option<TypeMap>,
    ) -> Output {
        let ret = executable
            .into_executable()
            .execute(self, &env.unwrap_or_default());
        self.apply_mutations();
        ret
    }
    pub fn execute_delayed_mutation<
        E: IntoExecutable<Args, Output = Output, Mutation = Mutation>,
        Args,
        Output,
        Mutation,
    >(
        &self,
        executable: E,
        env: Option<TypeMap>,
    ) -> (Output, Mutation) {
        executable
            .into_executable()
            .execute_delayed_mutation(self, &env.unwrap_or_default())
    }

    #[inline(always)]
    pub fn add_state<S: Any>(&mut self, state: S) -> &mut Self {
        self.states.insert(state);

        self
    }
    #[inline(always)]
    pub fn try_get_state_mut<S: Any>(&mut self) -> Option<&mut S> {
        self.states.get_mut()
    }
    #[inline(always)]
    pub fn get_state_mut<S: Any>(&mut self) -> &mut S {
        self.states.get_mut().unwrap_or_else(||
                panic!(
                    "Scaffolding error: Tried to load state of type `{}`, but it wasn't put in the world. Did you forget to load a plugin?",
                    core::any::type_name::<S>()
                )
            )
    }

    pub fn load_plugin<P: Plugin>(&mut self, mut plugin: P) -> &mut Self {
        if !self.plugins.contains::<P>() {
            plugin.load(self);
            self.plugins.insert(plugin);
        }

        self
    }

    #[inline(always)]
    pub fn queue_mutation(&self, mutation: Box<dyn UnsizedMutation>) {
        self.mutations.push(mutation);
    }
    pub fn apply_mutations(&mut self) {
        loop {
            // TODO: Find a more efficient iterator here
            // We need to iterate over elements from 0->mutations.len()
            // Need ownership over the elements, but removing each one
            // seems inefficient...
            let mutation = self.mutations.remove(0);
            match mutation {
                Some(mutation) => mutation.apply_unsized(self),
                None => break,
            }
        }
    }
}
impl Deref for World {
    type Target = ImmutableWorld;

    fn deref(&self) -> &Self::Target {
        &self.immut
    }
}
impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.immut
    }
}
impl Default for World {
    fn default() -> Self {
        Self::with_capacities(100, 10, 1_000, 100)
    }
}
