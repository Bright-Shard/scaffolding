#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

extern crate alloc;

pub mod datatypes;
#[cfg(feature = "multi")]
pub mod multi;
pub mod os;
pub mod utils;
pub mod world;

pub use ahash as _ahash;

pub mod prelude {
    //! Reexported types you'll probably need to use Scaffolding.

    #[cfg(feature = "multi")]
    pub use crate::multi::ExecuteInParallel;
    pub use crate::{
        datatypes::{uniq_key, TypeMap},
        world::{executable_args::*, ExecutableArg, Msg, State, World},
    };
}
pub mod plugin_prelude {
    //! Reexported types you'll probably need to make a Scaffolding plugin.

    pub use crate::prelude::*;
    pub use crate::{
        datatypes::{ArenaVec, StackVec, Uniq, Warehouse},
        world::{Executable, IntoExecutable, Plugin, UnsizedExecutable},
    };
}
