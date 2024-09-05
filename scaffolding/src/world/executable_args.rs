//! Types that can be used as arguments in [`Executable`]s.

use {
    crate::{datatypes::uniq::UniqKey, plugin_prelude::*},
    core::{
        fmt::{Debug, Formatter},
        ops::Deref,
    },
};

/// Types that can be used as arguments for [`Executable`]s.
pub trait ExecutableArg {
    type Arg<'a>: ExecutableArg;

    fn build(world: &World) -> Self::Arg<'_>;
    fn drop(self, world: &World);
}

// Included executable args below

/// Gets a singleton from the [`World`].
pub struct Singleton<'a, T: 'static> {
    val: &'a T,
}
impl<'a, T: 'static> Singleton<'a, T> {
    pub fn new(val: &'a T) -> Self {
        Self { val }
    }
}
impl<T: 'static> ExecutableArg for Singleton<'_, T> {
    type Arg<'a> = Singleton<'a, T>;

    fn build(world: &World) -> Self::Arg<'_> {
        Singleton {
            val: world.get_singleton(),
        }
    }
    fn drop(self, _: &World) {}
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

pub struct MsgSender<'a>(&'a World);
impl ExecutableArg for MsgSender<'_> {
    type Arg<'a> = MsgSender<'a>;

    fn build(world: &World) -> Self::Arg<'_> {
        MsgSender(world)
    }
    fn drop(self, _: &World) {}
}
impl MsgSender<'_> {
    pub fn send<M: 'static>(&self, msg: M) {
        self.0.send_msg(msg);
    }
}

pub struct Uniqs<'a>(&'a World);
impl ExecutableArg for Uniqs<'_> {
    type Arg<'a> = Uniqs<'a>;

    fn build(world: &World) -> Self::Arg<'_> {
        Uniqs(world)
    }
    fn drop(self, _: &World) {}
}
impl Uniqs<'_> {
    pub fn get<T: Default>(&self, key: UniqKey) -> &mut T {
        self.0.states.get_or_default(key)
    }
    pub fn get_or_insert<T>(&self, key: UniqKey, default: impl FnOnce() -> T) -> &mut T {
        self.0.states.get(key, default)
    }
}
