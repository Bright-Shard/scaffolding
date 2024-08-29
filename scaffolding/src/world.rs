//! Defines the [`World`], and types that interact with it.

pub mod executable;
pub mod executable_args;
pub mod plugin;

use {
    crate::datatypes::{
        typemap::{PubTypeId, TypeMap},
        uniq::Uniq,
        ArenaVec,
    },
    core::{
        any::Any,
        mem,
        ops::{Deref, DerefMut},
        ptr::NonNull,
        slice,
    },
};

pub use executable::*;
pub use executable_args::*;
pub use plugin::*;

pub struct Msg<M: 'static>(NonNull<M>);
impl<M: 'static> Deref for Msg<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}
impl<M: 'static> DerefMut for Msg<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}
impl<M: 'static> Msg<M> {
    pub fn read(self) -> M {
        unsafe { self.0.read() }
    }
}

pub struct World {
    pub plugins: TypeMap,
    pub singletons: TypeMap,
    pub states: Uniq,
    pub msg_handlers: TypeMap,
    msg_buffer: ArenaVec<u8>,
}
impl World {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacities(
        plugins: usize,
        singletons: usize,
        states: usize,
        msg_handlers: usize,
    ) -> Self {
        Self {
            plugins: TypeMap::new(plugins, 1_000),
            singletons: TypeMap::new(singletons, 1_000_000),
            states: Uniq::with_capacity(states),
            msg_handlers: TypeMap::new(msg_handlers, 1_000),
            msg_buffer: ArenaVec::default(),
        }
    }

    pub fn add_singleton<S: Any>(&mut self, state: S) -> &mut Self {
        self.singletons.insert(state);

        self
    }

    pub fn try_get_singleton<S: Any>(&self) -> Option<&S> {
        self.singletons.get()
    }
    pub fn try_get_singleton_mut<S: Any>(&mut self) -> Option<&mut S> {
        self.singletons.get_mut()
    }
    pub fn get_singleton<S: Any>(&self) -> &S {
        self.singletons.get().unwrap_or_else(||
                panic!(
                    "Scaffolding error: Tried to load state of type `{}`, but it wasn't put in the world. Did you forget to load a plugin?",
                    core::any::type_name::<S>()
                )
            )
    }
    pub fn get_singleton_mut<S: Any>(&mut self) -> &mut S {
        self.singletons.get_mut().unwrap_or_else(||
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

    /// Run an [`Executable`] with the data in this [`World`], then immediately
    /// apply any messages it sent.
    pub fn execute<'a, E: IntoExecutable<'a, Args>, Args>(&mut self, executable: E) -> E::Output {
        let out = executable.into_executable().execute(self);
        self.apply_msgs();
        out
    }
    /// Run an [`Executable`] with the data in this [`World`], but don't
    /// apply any messages it sent.
    pub fn execute_immut<'a, E: IntoExecutable<'a, Args>, Args>(&self, executable: E) -> E::Output {
        executable.into_executable().execute(self)
    }

    pub fn send_msg_now<M: 'static>(&mut self, mut msg: M) {
        if let Some(handler) = self.msg_handlers.get::<fn(&mut World, Msg<M>)>() {
            let ptr = unsafe { NonNull::new_unchecked(&mut msg as *mut M) };
            handler(self, Msg(ptr));
        }
        self.apply_msgs();
    }
    pub fn send_msg<M: 'static>(&self, msg: M) {
        // Message encoding:
        // - TypeId of `fn(&mut World, Msg<M>)`, so we can get the message
        // handler from the typemap later
        // - Size of the message's type
        // - Message

        let ty = PubTypeId::of::<fn(&mut World, Msg<M>)>();
        let ty = unsafe {
            slice::from_raw_parts(
                &ty as *const PubTypeId as *const u8,
                mem::size_of::<PubTypeId>(),
            )
        };
        self.msg_buffer.extend_from_slice(ty);

        let size = &(mem::size_of::<M>().to_ne_bytes());
        self.msg_buffer.extend_from_slice(size);

        let msg_bytes =
            unsafe { slice::from_raw_parts(&msg as *const M as *const u8, mem::size_of::<M>()) };
        self.msg_buffer.extend_from_slice(msg_bytes);

        // We "moved" the msg, so we don't want to run its drop function here,
        // if it has one
        mem::forget(msg);
    }
    pub fn add_msg_handler<M: 'static>(&mut self, handler: fn(&mut World, Msg<M>)) {
        self.msg_handlers.insert(handler);
    }
    pub fn apply_msgs(&mut self) {
        if self.msg_buffer.is_empty() {
            return;
        }

        let mut msg_buffer = ArenaVec::default();
        mem::swap(&mut self.msg_buffer, &mut msg_buffer);

        // See the comment in [`Self::queue_msg`] for the format we decode here
        let mut current_msg = msg_buffer.as_mut_slice();
        loop {
            let ty_ptr = current_msg as *const [u8] as *const u8 as *const PubTypeId;
            let ty = unsafe { ty_ptr.read() };

            let size_ptr = &current_msg[mem::size_of::<PubTypeId>()..] as *const [u8] as *const u8
                as *const usize;
            let size = unsafe { size_ptr.read() };

            if let Some(handler) = self.msg_handlers.get_raw(ty) {
                let msg_ptr = &mut current_msg
                    [mem::size_of::<PubTypeId>() + mem::size_of::<usize>()..]
                    as *mut [u8] as *mut u8;

                let msg_handler: NonNull<fn(&mut World, Msg<u8>)> = handler.cast();

                unsafe { (msg_handler.as_ref())(self, Msg(NonNull::new_unchecked(msg_ptr))) }
            }

            let msg_len = mem::size_of::<PubTypeId>() + mem::size_of::<usize>() + size;

            if current_msg.len() <= msg_len {
                break;
            }

            current_msg = &mut current_msg[msg_len..];
        }
    }
}
impl Default for World {
    fn default() -> Self {
        Self::with_capacities(100, 1_000, 100, 100)
    }
}
