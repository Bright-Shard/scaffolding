#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

extern crate alloc;

pub mod datatypes;
#[cfg(feature = "multi")]
pub mod multi;
pub mod os;
pub mod utils;
pub mod world;

pub mod prelude {
    //! Reexported types you'll probably need to use Scaffolding.

    pub use crate::{
        datatypes::TypeMap,
        world::{executable_args::*, ExecutableArg, Mutation, World},
    };
}
pub mod plugin_prelude {
    //! Reexported types you'll probably need to make a Scaffolding plugin.

    pub use crate::{
        datatypes::{ArenaVec, StackVec, TypeMap, Warehouse},
        world::{
            executable_args::*, Executable, ExecutableArg, ImmutableWorld, IntoExecutable,
            Mutation, MutationSet, Plugin, UnsizedExecutable, UnsizedMutation, World,
        },
    };
}

use {
    core::{
        ptr::addr_of_mut,
        sync::atomic::{AtomicBool, Ordering},
    },
    os::OsMetadata,
};

/// Used by Scaffolding to lazily initialize global state.
static INTIALISED: AtomicBool = AtomicBool::new(false);

/// Initializes global Scaffolding variables. This will be called for you when
/// a new [`World`] is created.
///
/// [`World`]: world::World
pub fn init() {
    if !INTIALISED.load(Ordering::Relaxed) {
        unsafe {
            *addr_of_mut!(os::OS_INFO) = OsMetadata::default();
        }

        INTIALISED.store(true, Ordering::Release);
    }
}
