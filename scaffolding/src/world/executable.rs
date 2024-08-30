use {
    crate::{
        self as scaffolding,
        datatypes::uniq::{uniq_key, UniqKey},
        world::{ExecutableArg, World},
    },
    alloc::boxed::Box,
    core::{
        any::Any,
        cell::UnsafeCell,
        marker::PhantomData,
        ops::{Deref, DerefMut, Fn},
    },
};

/// Executables are functions that get data from the [`World`].
pub trait Executable<'a>: Sized + 'a {
    /// The return type.
    type Output: 'a;

    /// Run this executable in the give [`World`]. Note that this will not apply
    /// any messages that this executable sent to the world; use
    /// [`World::apply_msgs`] for that.
    fn execute(&self, world: &World) -> Self::Output;
}

/// A version of [`Executable`] that can be used as a trait object. Normal
/// executables must be [`Sized`], and have associated types for their output
/// and arguments. This type has none of those requirements, allowing it to be
/// simply stored as a `Box<dyn UnsizedExecutable>`. It's also automatically
/// implemented for all executables, which means any executable can be upcasted
/// to a trait object with this trait.
///
/// Because this trait doesn't have associated type, it relies heavily on trait
/// objects and dynamic typing, which adds overhead. Thus, [`Executable`]
/// should be preferred and used where possible.
pub trait UnsizedExecutable<'a>: 'a {
    /// The same as [`Executable::execute`], except this trait doesn't have an
    /// associated `Output` type, so the output has to be returned as an
    /// [`Any`] trait object.
    fn execute_unsized(&self, world: &World) -> Box<dyn Any>;
}
impl<'a, E, Output> UnsizedExecutable<'a> for E
where
    Output: 'static,
    E: Executable<'a, Output = Output>,
{
    fn execute_unsized(&self, world: &World) -> Box<dyn Any> {
        Box::new(self.execute(world))
    }
}

/// A borrowed [`ExecutableArg`].
///
/// This trait is implemented for `&T` and `&mut T`, where `T: ExecutableArg`.
/// It allows creating either borrow (`&T` or `&mut T`) from an `&mut T`. This
/// allows executables to take `&ExecutableArg` or `&mut ExecutableArg` in
/// their arguments.
///
/// # Why this trait is needed
///
/// Note: The following documentation is only relevant if you want to
/// understand exactly how Scaffolding works. If you're just trying to use the
/// library, you don't need to know this - it's pretty technicaly internal
/// details.
///
/// Executable args can only be borrowed by executables, not owned. This is
/// needed so that the executable arg can be passed to the executable without
/// being dropped, because we need to run their dedicated destructors after the
/// executable runs (see [`ExecutableArg::drop`]).
///
/// This creates a new issue, however, because now executables can take both
/// mutable and immutable borrows, and those can be in any order - like these
/// two executables:
///
/// ```rust
/// use scaffolding::prelude::*;
///
/// fn executable(int: &Singleton<i32>, float: &mut Singleton<f32>) {
///   // Some very important code here
/// }
///
/// fn also_an_executable(float: &mut Singleton<f32>, int: &Singleton<i32>) {
///   // Some very important code here
/// }
/// ```
///
/// These two functions are almost identical, they just have their arguments
/// flipped. The issue is that they now have two different types - the first is
/// a `Fn(&ExecutableArg, &mut ExectableArg)`, but the second is a
/// `Fn(&mut ExecutableArg, &ExectableArg)`. We have to cover both cases when
/// implementing [`IntoExecutable`] for functions - and this problem only gets
/// worse as we add more arguments to functions. Here's the types for
/// executables with just two arguments:
///
/// - `Fn(&ExecutableArg)`
/// - `Fn(&mut ExecutableArg)`
/// - `Fn(&ExecutableArg, &ExecutableArg)`
/// - `Fn(&mut ExecutableArg, &mut ExecutableArg)`
/// - `Fn(&ExecutableArg, &mut ExecutableArg)`
/// - `Fn(&mut ExecutableArg, &ExecutableArg)`
///
/// We could use macros to generate implementations for all these types, but
/// we'd end up generating hundreds of lines of code, and it'd be pretty hard
/// to implement.
///
/// This is where [`ExecutableArgRef`] comes in. It's implemented for both
/// `&ExecutableArg` *and* `&mut ExecutableArg` - which means we can now
/// represent all the above functions as simply:
///
/// - `Fn(ExecutableArgRef)`
/// - `Fn(ExecutableArgRef, ExecutableArgRef)`
///
/// We can also then create the correct borrow type (`&` or `&mut`) using
/// [`ExecutableArgRef::borrow`]. This means we don't have to generate code for
/// every single combination of `&`/`&mut` in executables.
pub trait ExecutableArgRef {
    /// The [`ExecutableArg`] being borrowed.
    type EA: ExecutableArg;
    /// The borrowed executable arg's type - either `&ExecutableArg` or
    /// `&mut ExecutableArg`. This has two lifetimes: `'a`, the lifetime of the
    /// [`ExecutableArg`], and `'b`, the lifetime of the borrow.
    type Borrowed<'a: 'b, 'b>
    where
        <Self::EA as ExecutableArg>::Arg<'a>: 'a;

    /// Creates the [`Self::Borrowed`] type from an `&mut ExecutableArg`.
    /// [`Self::Borrowed`] is either `&ExecutableArg` or `&mut ExecutableArg`,
    /// so this allows converting an `&mut ExecutableArg` into either borrow
    /// type.
    fn borrow<'a: 'b, 'b>(
        from: &'b mut <Self::EA as ExecutableArg>::Arg<'a>,
    ) -> Self::Borrowed<'a, 'b>
    where
        <Self::EA as ExecutableArg>::Arg<'a>: 'a;
}
impl<EA: ExecutableArg> ExecutableArgRef for &EA {
    type EA = EA;
    type Borrowed<'a: 'b, 'b> = &'b EA::Arg<'a> where EA::Arg<'a>: 'a;

    #[inline(always)]
    fn borrow<'a: 'b, 'b>(
        from: &'b mut <Self::EA as ExecutableArg>::Arg<'a>,
    ) -> Self::Borrowed<'a, 'b>
    where
        EA::Arg<'a>: 'a,
    {
        from
    }
}
impl<EA: ExecutableArg> ExecutableArgRef for &mut EA {
    type EA = EA;
    type Borrowed<'a: 'b, 'b> = &'b mut EA::Arg<'a> where EA::Arg<'a>: 'a;

    #[inline(always)]
    fn borrow<'a: 'b, 'b>(
        from: &'b mut <Self::EA as ExecutableArg>::Arg<'a>,
    ) -> Self::Borrowed<'a, 'b>
    where
        EA::Arg<'a>: 'a,
    {
        from
    }
}

pub trait IntoExecutable<'a, Args>: 'a {
    type Output: 'a;
    type Executable: Executable<'a, Output = Self::Output> + 'a;

    fn into_executable(self) -> Self::Executable;
}
impl<'a, E: Executable<'a> + 'a> IntoExecutable<'a, ()> for E {
    type Output = E::Output;
    type Executable = E;

    fn into_executable(self) -> Self::Executable {
        self
    }
}

pub trait IntoStatefulExecutable<'a, Args> {
    type Output: 'a;
    type State: 'a;
    type KeyedExecutable: Executable<'a, Output = Self::Output> + 'a
    where
        Self::State: Default;
    type StateExecutable: Executable<'a, Output = Self::Output> + 'a;

    fn into_executable_with_key(self, key: UniqKey) -> Self::KeyedExecutable
    where
        Self::State: Default;
    fn into_executable_with_state(self, state: Self::State) -> Self::StateExecutable;
    fn run_with_state(self, state: &mut Self::State, world: &World) -> Self::Output;
}

#[repr(transparent)]
pub struct StatelessExecutable<'a, Func: 'a, Args>(pub Func, pub PhantomData<&'a Args>);

#[repr(transparent)]
pub struct State<'a, T>(pub &'a mut T);
impl<T> Deref for State<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<T> DerefMut for State<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

pub struct StatefulExecutableKeyed<'a, Func: 'a, Args, S: Default + 'a> {
    pub func: Func,
    pub key: UniqKey,
    pub _args: PhantomData<&'a Args>,
    pub _state: PhantomData<S>,
}
pub struct StatefulExecutableState<'a, Func: 'a, Args, S: 'a> {
    pub func: Func,
    pub state: UnsafeCell<S>,
    pub _args: PhantomData<&'a Args>,
}

// TODO: Use the below macro to implement executable for functions, like so:
// impl_executable!(A ARef A ARef B BRef C CRef D DRef E ERef F FRef);
// This is currently held back by a compiler bug:
// https://github.com/rust-lang/rust/issues/100013
// I've subscribed to the issue and should get notified when it gets fixed. For
// now a temporary workaround is implemented below this macro below.
macro_rules! impl_executable {
    // No Arguments
    ($_unused:ident $_unused2:ident) => {
        impl<'a, Function, Output> IntoExecutable<'a, ((),)> for Function
        where
            Output: 'a,
            Function: 'a,
            Function: Fn() -> Output + 'a,
        {
            type Executable = StatelessExecutable<'a, Self, ()>;
            type Output = Output;

            fn into_executable(self) -> Self::Executable {
                StatelessExecutable(self, PhantomData)
            }
        }
        impl<'a, Function, Output, StateTy> IntoStatefulExecutable<'a, (StateTy,)> for Function
        where
            StateTy: 'a,
            Output: 'a,
            Function: 'a,
            Function: Fn(State<'_, StateTy>) -> Output + 'a,
        {
            type Output = Output;
            type State = StateTy;
            type KeyedExecutable = StatefulExecutableKeyed<'a, Self, (), StateTy>
            where
                Self::State: Default;
            type StateExecutable = StatefulExecutableState<'a, Self, (), StateTy>;

            fn into_executable_with_key(self, key: UniqKey) -> Self::KeyedExecutable
            where
                Self::State: Default
            {
                StatefulExecutableKeyed {
                    func: self,
                    key,
                    _args: PhantomData,
                    _state: PhantomData
                }
            }
            fn into_executable_with_state(self, state: Self::State) -> Self::StateExecutable {
                StatefulExecutableState {
                    func: self,
                    state: UnsafeCell::new(state),
                    _args: PhantomData
                }
            }
            fn run_with_state(self, state: &mut Self::State, _: &World) -> Self::Output {
                (self)(State(state))
            }
        }

        impl<'a, Function, Output> Executable<'a> for StatelessExecutable<'a, Function, ()>
        where
            Output: 'a,
            Function: Fn() -> Output + 'a
        {
            type Output = Output;

            fn execute(&self, _: &World) -> Self::Output {
                self.0()
            }
        }
        impl<'a, Function, Output, StateTy> Executable<'a> for StatefulExecutableKeyed<'a, Function, (), StateTy>
        where
            StateTy: Default + 'a,
            Output: 'a,
            Function: Fn(State<'_, StateTy>) -> Output + 'a
        {
            type Output = Output;

            fn execute(&self, world: &World) -> Self::Output {
                let state = world.states.get_or_default(uniq_key!((&self.key.as_modifier())));
                (self.func)(State(state))
            }
        }
        impl<'a, Function, Output, StateTy> Executable<'a> for StatefulExecutableState<'a, Function, (), StateTy>
        where
            StateTy: 'a,
            Output: 'a,
            Function: Fn(State<'_, StateTy>) -> Output + 'a
        {
            type Output = Output;

            fn execute(&self, _: &World) -> Self::Output {
                (self.func)(State(unsafe { &mut *self.state.get() }))
            }
        }
    };

    // Arguments
    ($_unused:ident $_unused2:ident $($ty:ident $tyref:ident)*) => {
        impl<Function, Output, $($ty),*, $($tyref),*> IntoExecutable<($($ty),*, $($tyref),*)> for Function
        where
            $(for<'a> $ty: ExecutableArg + 'a),*,
            $(for<'a> $tyref: ExecutableArgRef<EA = $ty> + 'a),*,
            Output: 'static,
            Function: 'static,
            Function: Fn($($tyref::Borrowed<'_, '_>),*) -> Output,
            Function: Fn($($tyref),*) -> Output,
        {
            type Executable = StatelessExecutable<Self, ($($ty),*, $($tyref),*)>;
            type Output = Output;
            #[allow(unused_parens)]
            type Mutation = ($($ty::Mutation),*);

            fn into_executable(self) -> Self::Executable {
                StatelessExecutable(self, PhantomData)
            }
        }

        impl<Function, Output, $($ty),*, $($tyref),*> Executable for StatelessExecutable<Function, ($($ty),*, $($tyref),*)>
        where
            $(for<'a> $ty: ExecutableArg + 'a),*,
            $(for<'a> $tyref: ExecutableArgRef<EA = $ty> + 'a),*,
            Output: 'static,
            Function: 'static,
            Function: Fn($($tyref::Borrowed<'_, '_>),*) -> Output,
            Function: Fn($($tyref),*) -> Output,
        {
            type Output = Output;
            #[allow(unused_parens)]
            type Mutation = ($($ty::Mutation),*);

            #[allow(non_snake_case)]
            fn execute(&self, world: &mut World, env: &TypeMap) -> Self::Output {
                $(let mut $ty = $ty::from_world_and_env(world, env);)*
                let result = self.0($($tyref::borrow(&mut $ty)),*);
                $(let $ty = $ty.build_mutation();)*
                $($ty.apply(world);)*

                result
            }
            #[allow(non_snake_case)]
            fn execute_delayed_mutation(&self, world: &World, env: &TypeMap) -> (Self::Output, Self::Mutation) {
                $(let mut $ty = $ty::from_world_and_env(world, env);)*
                let result = self.0($($tyref::borrow(&mut $ty)),*);

                (result, ($($ty.build_mutation()),*))
            }
        }

        impl_executable!($($ty $tyref)*);
    };
}
// impl_executable!(A ARef A ARef B BRef C CRef D DRef E ERef F FRef);

// This is the workaround implementation of `Executable`. It uses simpler
// lifetimes that the compiler will understand, but are actually much longer
// than they should be, so we have to unsafely create borrows with excessively
// long lifetimes from pointers to get this implementation working.
macro_rules! impl_executable_workaround {
    // No Arguments
    ($_unused:ident $_unused2:ident) => {
        // Triggers the no arguments branch of `impl_executable`, which is
        // valid / insn't held back by the compiler issue.
        impl_executable!(A A);
    };

    // Arguments
    ($_unused:ident $_unused2:ident $($ty:ident $tyref:ident)*) => {
        impl<'a, Function, Output, $($ty),*, $($tyref),*> IntoExecutable<'a, ($($ty),*, $($tyref),*)> for Function
        where
            $($ty: ExecutableArg + 'a),*,
            $($tyref: ExecutableArgRef<EA = $ty> + 'a),*,
            Output: 'a,
            Function: 'a,
            Function: for<'b> Fn($($tyref::Borrowed<'b, 'b>),*) -> Output,
            Function: Fn($($tyref),*) -> Output,
        {
            type Executable = StatelessExecutable<'a, Self, ($($ty),*, $($tyref),*)>;
            type Output = Output;

            fn into_executable(self) -> Self::Executable {
                StatelessExecutable(self, PhantomData)
            }
        }
        impl<'a, Function, Output, StateTy, $($ty),*, $($tyref),*> IntoStatefulExecutable<'a, (StateTy, $($ty),*, $($tyref),*)> for Function
        where
            $($ty: ExecutableArg + 'a),*,
            $($tyref: ExecutableArgRef<EA = $ty> + 'a),*,
            StateTy: 'a,
            Output: 'a,
            Function: 'a,
            Function: for<'b> Fn(State<'_, StateTy>, $($tyref::Borrowed<'b, 'b>),*) -> Output,
            Function: Fn(State<'_, StateTy>, $($tyref),*) -> Output,
        {
            type Output = Output;
            type State = StateTy;
            type KeyedExecutable = StatefulExecutableKeyed<'a, Self, ($($ty),*, $($tyref),*), StateTy>
            where
                Self::State: Default;
            type StateExecutable = StatefulExecutableState<'a, Self, ($($ty),*, $($tyref),*), StateTy>;

            fn into_executable_with_key(self, key: UniqKey) -> Self::KeyedExecutable
            where
                Self::State: Default {
                StatefulExecutableKeyed {
                    func: self,
                    key,
                    _args: PhantomData,
                    _state: PhantomData
                }
            }
            fn into_executable_with_state(self, state: Self::State) -> Self::StateExecutable {
                StatefulExecutableState {
                    func: self,
                    state: UnsafeCell::new(state),
                    _args: PhantomData
                }
            }
            #[allow(non_snake_case)]
            fn run_with_state(self, state: &mut Self::State, world: &World) -> Self::Output {
                let world_extended: &World = unsafe { &*(world as *const World) };

                $(let mut $ty = $ty::build(world_extended);)*

                let result = (self)(
                    State(state),
                    $($tyref::borrow(unsafe { &mut *(&mut $ty as *mut $ty::Arg<'_>) })),*
                );
                $($ty.drop(world);)*

                result
            }
        }

        impl<'a, Function, Output, $($ty),*, $($tyref),*> Executable<'a> for StatelessExecutable<'a, Function, ($($ty),*, $($tyref),*)>
        where
            $($ty: ExecutableArg + 'a),*,
            $($tyref: ExecutableArgRef<EA = $ty> + 'a),*,
            Output: 'a,
            Function: 'a,
            Function: for<'b> Fn($($tyref::Borrowed<'b, 'b>),*) -> Output,
            Function: Fn($($tyref),*) -> Output,
        {
            type Output = Output;

            #[allow(non_snake_case)]
            fn execute(&self, world: &World) -> Self::Output {
                let world_extended: &World = unsafe { &*(world as *const World) };

                $(let mut $ty = $ty::build(world_extended);)*
                let result = self.0($($tyref::borrow(unsafe { &mut *(&mut $ty as *mut $ty::Arg<'_>) })),*);
                $($ty.drop(world);)*

                result
            }
        }
        impl<'a, Function, Output, StateTy, $($ty),*, $($tyref),*> Executable<'a> for StatefulExecutableKeyed<'a, Function, ($($ty),*, $($tyref),*), StateTy>
        where
            $($ty: ExecutableArg + 'a),*,
            $($tyref: ExecutableArgRef<EA = $ty> + 'a),*,
            StateTy: Default + 'a,
            Output: 'a,
            Function: 'a,
            Function: for<'b> Fn(State<'_, StateTy>, $($tyref::Borrowed<'b, 'b>),*) -> Output,
            Function: Fn(State<'_, StateTy>, $($tyref),*) -> Output,
        {
            type Output = Output;

            #[allow(non_snake_case)]
            fn execute(&self, world: &World) -> Self::Output {
                let world_extended: &World = unsafe { &*(world as *const World) };

                $(let mut $ty = $ty::build(world_extended);)*
                let state = world.states.get_or_default(unsafe { self.key.clone() });

                let result = (self.func)(
                    State(state),
                    $($tyref::borrow(unsafe { &mut *(&mut $ty as *mut $ty::Arg<'_>) })),*
                );
                $($ty.drop(world);)*

                result
            }
        }
        impl<'a, Function, Output, StateTy, $($ty),*, $($tyref),*> Executable<'a> for StatefulExecutableState<'a, Function, ($($ty),*, $($tyref),*), StateTy>
        where
            $($ty: ExecutableArg + 'a),*,
            $($tyref: ExecutableArgRef<EA = $ty> + 'a),*,
            StateTy: 'a,
            Output: 'a,
            Function: 'a,
            Function: for<'b> Fn(State<'_, StateTy>, $($tyref::Borrowed<'b, 'b>),*) -> Output,
            Function: Fn(State<'_, StateTy>, $($tyref),*) -> Output,
        {
            type Output = Output;

            #[allow(non_snake_case)]
            fn execute(&self, world: &World) -> Self::Output {
                let world_extended: &World = unsafe { &*(world as *const World) };

                $(let mut $ty = $ty::build(world_extended);)*

                let result = (self.func)(
                    State(unsafe { &mut *self.state.get() }),
                    $($tyref::borrow(unsafe { &mut *(&mut $ty as *mut $ty::Arg<'_>) })),*
                );
                $($ty.drop(world);)*

                result
            }
        }

        impl_executable_workaround!($($ty $tyref)*);
    };
}
impl_executable_workaround!(A ARef A ARef B BRef C CRef D DRef E ERef F FRef);

#[cfg(test)]
mod tests {
    use {super::*, crate::prelude::*};

    fn accepts_executable<'a, Args>(func: impl IntoExecutable<'a, Args>) {
        let mut world = World::new();
        world.add_singleton(0_u32);
        world.add_singleton(1_i32);
        world.execute(func);
    }

    fn executable(_num: &mut Singleton<i32>) {}
    fn executable2(_num: &Singleton<i32>, _num2: &mut Singleton<u32>) {}
    fn executable3() {}

    fn stateful_executable(_state: State<u32>) {}
    fn stateful_executable2(_state: State<u32>, _num: &mut Singleton<u32>) {}

    #[test]
    fn type_test() {
        accepts_executable(executable);
        accepts_executable(executable2);
        accepts_executable(executable3);

        accepts_executable(|| {});
        accepts_executable(|_: &mut Singleton<i32>| {});
        accepts_executable(|_: &Singleton<i32>, _: &mut Singleton<i32>| {});

        accepts_executable(stateful_executable.into_executable_with_key(uniq_key!()));
        accepts_executable(stateful_executable2.into_executable_with_key(uniq_key!()));
    }
}
