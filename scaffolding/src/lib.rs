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

    #[cfg(feature = "multi")]
    pub use crate::multi::ExecuteInParallel;
    pub use crate::{
        datatypes::TypeMap,
        world::{executable_args::*, ExecutableArg, Mutation, World},
    };
}
pub mod plugin_prelude {
    //! Reexported types you'll probably need to make a Scaffolding plugin.

    #[cfg(feature = "multi")]
    pub use crate::multi::ExecuteInParallel;
    pub use crate::{
        datatypes::{ArenaVec, StackVec, TypeMap, Warehouse},
        world::{
            executable_args::*, Executable, ExecutableArg, ImmutableWorld, IntoExecutable,
            Mutation, MutationSet, Plugin, UnsizedExecutable, UnsizedMutation, World,
        },
    };
}
