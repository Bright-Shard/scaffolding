//! Defines the [`World`], and types that interact with it.

pub mod executable;
pub mod executable_args;
pub mod plugin;

pub use executable::*;
pub use executable_args::*;
pub use plugin::*;

use {crate::datatypes::TypeMap, core::any::Any};

pub struct World {
    pub plugins: TypeMap,
    pub states: TypeMap,
    pub msg_handlers: TypeMap,
}
impl World {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacities(
        num_states: usize,
        num_plugins: usize,
        num_event_handlers: usize,
        states_memory: usize,
        plugins_memory: usize,
        event_handlers_memory: usize,
    ) -> Self {
        Self {
            plugins: TypeMap::new(num_plugins, plugins_memory),
            states: TypeMap::new(num_states, states_memory),
            msg_handlers: TypeMap::new(num_event_handlers, event_handlers_memory),
        }
    }

    pub fn add_singleton<S: Any>(&mut self, state: S) -> &mut Self {
        self.states.insert(state);

        self
    }

    pub fn try_get_singleton<S: Any>(&self) -> Option<&S> {
        self.states.get()
    }
    pub fn try_get_singleton_mut<S: Any>(&mut self) -> Option<&mut S> {
        self.states.get_mut()
    }
    pub fn get_singleton<S: Any>(&self) -> &S {
        self.states.get().unwrap_or_else(||
                panic!(
                    "Scaffolding error: Tried to load state of type `{}`, but it wasn't put in the world. Did you forget to load a plugin?",
                    core::any::type_name::<S>()
                )
            )
    }
    pub fn get_singleton_mut<S: Any>(&mut self) -> &mut S {
        self.states.get_mut().unwrap_or_else(||
        panic!(
            "Scaffolding error: Tried to load state of type `{}`, but it wasn't put in the world. Did you forget to load a plugin?",
            core::any::type_name::<S>()
        )
    )
    }

    pub fn add_plugin<P: Plugin>(&mut self, mut plugin: P) -> &mut Self {
        if !self.plugins.contains::<P>() {
            plugin.load(self);
            self.plugins.insert(plugin);
        }

        self
    }
    pub fn has_plugin<P: Plugin>(&self) -> bool {
        self.plugins.contains::<P>()
    }

    /// Run an [`Executable`] with the data in this [`World`].
    pub fn execute<E: IntoExecutable<Args, Output = Output>, Args, Output>(
        &mut self,
        executable: E,
    ) -> Output {
        executable.into_executable().execute(self)
    }

    pub fn send_msg<M: 'static>(&mut self, msg: M) {
        if let Some(handler) = self.msg_handlers.get::<fn(&mut World, M)>() {
            handler(self, msg);
        }
    }
    pub fn set_msg_handler<M: 'static>(&mut self, handler: fn(&mut World, M)) {
        self.msg_handlers.insert(handler);
    }
}
impl Default for World {
    fn default() -> Self {
        Self::with_capacities(100, 10, 100, 1_000, 100, 1_600)
    }
}
