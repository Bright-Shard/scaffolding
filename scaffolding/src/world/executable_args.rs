//! Standard executable args provided by Scaffolding.

use {
    crate::{
        datatypes::TypeMap,
        world::{ExecutableArg, ImmutableWorld},
    },
    core::ops::Deref,
};

pub struct State<'a, T: 'static> {
    val: &'a T,
}
impl<T: 'static> ExecutableArg for State<'_, T> {
    type Arg<'a> = State<'a, T>;
    type Mutation = ();

    fn from_world_and_env<'a>(world: &'a ImmutableWorld, _: &'a TypeMap) -> Self::Arg<'a> {
        State {
            val: world.get_state(),
        }
    }
    fn build_mutation(self) -> Self::Mutation {}
}
impl<'a, T> Deref for State<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val
    }
}

pub struct Env<'a, T: 'static> {
    val: &'a T,
}
impl<T: 'static> ExecutableArg for Env<'_, T> {
    type Arg<'a> = State<'a, T>;
    type Mutation = ();

    fn from_world_and_env<'a>(_: &'a ImmutableWorld, env: &'a TypeMap) -> Self::Arg<'a> {
        State {
            val: env.get().unwrap_or_else(|| {
                panic!(
                    "Env needed type {}, but the environment didn't have it",
                    core::any::type_name::<T>(),
                )
            }),
        }
    }
    fn build_mutation(self) -> Self::Mutation {}
}
impl<'a, T> Deref for Env<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val
    }
}
