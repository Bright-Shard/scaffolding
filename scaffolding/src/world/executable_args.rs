//! Types that can be used as arguments in [`Executable`]s.

use {
    crate::{prelude::TypeMap, world::World},
    core::{
        fmt::{Debug, Formatter},
        ops::Deref,
    },
};

/// Types that can be used as arguments for [`Executable`]s.
pub trait ExecutableArg {
    type Arg<'a>: ExecutableArg;

    fn build(world: &World) -> Self::Arg<'_>;
    fn on_drop(self) -> impl FnOnce(&mut World) + Send + 'static;
}

// Included executable args below

/// Gets a singleton from the [`World`].
pub struct Singleton<'a, T: 'static> {
    val: &'a T,
}
impl<T: 'static> ExecutableArg for Singleton<'_, T> {
    type Arg<'a> = Singleton<'a, T>;

    fn build(world: &World) -> Self::Arg<'_> {
        Singleton {
            val: world.get_singleton(),
        }
    }
    fn on_drop(self) -> impl FnOnce(&mut World) + Send + 'static {
        |_| {}
    }
}
impl<'a, T> Deref for Singleton<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val
    }
}
impl<T: Debug> Debug for Singleton<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "State {{ val: {:?} }}", self.val)
    }
}

/// Sets a callback to run after the current [`Executable`] finishes running.
pub struct PostRunCallback<F: FnOnce(&mut World) + Send + 'static>(Option<F>);
impl<F: FnOnce(&mut World) + Send + 'static> PostRunCallback<F> {
    pub fn set_callback(&mut self, func: F) {
        self.0 = Some(func);
    }
}
impl<F: FnOnce(&mut World) + Send + 'static> ExecutableArg for PostRunCallback<F> {
    type Arg<'a> = Self;

    fn build(_: &World) -> Self::Arg<'_> {
        Self(None)
    }
    fn on_drop(self) -> impl FnOnce(&mut World) + Send + 'static {
        move |world| {
            if let Some(cb) = self.0 {
                cb(world)
            }
        }
    }
}
