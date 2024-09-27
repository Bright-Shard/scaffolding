#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

extern crate alloc;

pub mod datatypes;
pub mod os;
pub mod utils;
pub mod world;

pub mod _hash {
    #[cfg(feature = "ahash")]
    pub use ahash::AHasher as Hasher;
    #[cfg(feature = "std")]
    pub use std::hash::DefaultHasher as Hasher;
    #[cfg(all(not(feature = "std"), not(feature = "ahash")))]
    compile_error!("You must enable either the `std` or `ahash` features.");
}

pub mod prelude {
    //! Reexported types you'll probably need to use Scaffolding.

    pub use crate::{
        datatypes::{uniq_key, TypeMap},
        world::{
            executable_args::*, DynamicExecutable as _, Executable as _, ExecutableArg,
            ExecutableWithState as _, Msg, TypeErasedExecutable as _, World,
        },
    };
}
pub mod plugin_prelude {
    //! Reexported types you'll probably need to make a Scaffolding plugin.

    pub use crate::prelude::*;
    pub use crate::{
        datatypes::{ArenaVec, StackVec, Uniq, Warehouse},
        world::{DynamicExecutable, Executable, ExecutableWithState, Plugin, TypeErasedExecutable},
    };
}
