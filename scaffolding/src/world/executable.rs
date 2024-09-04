use {
    crate::world::{ExecutableArg, World},
    alloc::boxed::Box,
    core::{any::Any, marker::PhantomData},
};

/// Executables are functions that get data from the [`World`].
///
/// Executables are a special subset of functions whose arguments are only
/// `&impl ExecutableArg` or `&mut ExecutableArg` - that is, there arguments
/// are only borrows of types that implement `ExecutableArg`.
pub trait Executable<'a, Args: 'a>: Sized + 'a {
    /// The return type.
    type Output: 'a;

    /// Run this executable in the given [`World`].
    ///
    /// Note that, unlike [`World::execute`], this does not automatically
    /// process messages sent to the [`World`]; you'll need to call
    /// [`World::process_msgs`] separately to do that.
    fn execute(self, world: &World) -> Self::Output;
    /// Convert this [`Executable`] into a [`TypeErasedExecutable`].
    fn type_erase(self) -> impl TypeErasedExecutable<'a, Output = Self::Output> {
        ExecutableStore {
            executable: self,
            _ph: PhantomData,
        }
    }
    /// Convert this [`Executable`] into a [`DynamicExecutable`].
    fn make_dynamic(self) -> impl DynamicExecutable
    where
        'a: 'static,
    {
        ExecutableStore {
            executable: self,
            _ph: PhantomData,
        }
    }
}
/// The same as [`Executable`], except this trait doesn't store its argument
/// type in a generic. You can convert an [`Executable`] into this with
/// [`Executable::type_erase`].
pub trait TypeErasedExecutable<'a>: 'a {
    type Output: 'a;

    /// Run this executable in the given [`World`].
    ///
    /// Note that, unlike [`World::execute`], this does not automatically
    /// process messages sent to the [`World`]; you'll need to call
    /// [`World::process_msgs`] separately to do that.
    fn execute(self, world: &World) -> Self::Output;
    /// Convert this [`TypeErasedExecutable`] into a [`DynamicExecutable`].
    fn make_dynamic(self) -> impl DynamicExecutable
    where
        'a: 'static;
}
/// The same as [`Executable`], except this trait doesn't have any generics
/// or associated types. Thus, it can be used as a trait object. You can create
/// one of these with [`Executable::make_dynamic`].
pub trait DynamicExecutable {
    /// Run this executable in the given [`World`].
    ///
    /// Note that, unlike [`World::execute`], this does not automatically
    /// process messages sent to the [`World`]; you'll need to call
    /// [`World::process_msgs`] separately to do that.
    ///
    /// In addition, unlike [`Executable::execute`], this type doesn't store
    /// its output type, so it returns a `Box<dyn Any>`.
    fn execute(self, world: &World) -> Box<dyn Any>;
}
/// An [`Executable`] with a custom first argument.
pub trait ExecutableWithState<'a, State: 'a, Args: 'a>: Sized + 'a {
    type Output: 'a;

    fn with_state(self, state: State) -> impl Executable<'a, Args, Output = Self::Output>;
    /// Run this executable in the given [`World`] with the given state.
    ///
    /// Note that, unlike [`World::execute`], this does not automatically
    /// process messages sent to the [`World`]; you'll need to call
    /// [`World::process_msgs`] separately to do that.
    fn execute(self, state: State, world: &World) -> Self::Output;
}

/// Wraps around an executable to type-erase it.
pub struct ExecutableStore<'a, Args, E: Executable<'a, Args>> {
    pub executable: E,
    pub _ph: PhantomData<&'a Args>,
}
impl<'a, Args, E: Executable<'a, Args>> Executable<'a, Args> for ExecutableStore<'a, Args, E> {
    type Output = E::Output;

    fn execute(self, world: &World) -> Self::Output {
        self.executable.execute(world)
    }
    fn make_dynamic(self) -> impl DynamicExecutable
    where
        'a: 'static,
    {
        self
    }
}
impl<'a, Args, E: Executable<'a, Args>> TypeErasedExecutable<'a> for ExecutableStore<'a, Args, E> {
    type Output = E::Output;

    fn execute(self, world: &World) -> Self::Output {
        self.executable.execute(world)
    }
    fn make_dynamic(self) -> impl DynamicExecutable
    where
        'a: 'static,
    {
        self
    }
}
impl<Args, E: Executable<'static, Args>> DynamicExecutable for ExecutableStore<'static, Args, E> {
    fn execute(self, world: &World) -> Box<dyn Any> {
        Box::new(self.executable.execute(world))
    }
}

/// Wraps around an executable with a state to type-erase it and store its
/// state.
pub struct ExecutableWithStateStore<'a, State: 'a, Args, E: ExecutableWithState<'a, State, Args>> {
    pub executable: E,
    pub state: State,
    pub _ph: PhantomData<&'a Args>,
}
impl<'a, Args, State: 'a, E: ExecutableWithState<'a, State, Args>> Executable<'a, Args>
    for ExecutableWithStateStore<'a, State, Args, E>
{
    type Output = E::Output;

    fn execute(self, world: &World) -> Self::Output {
        self.executable.execute(self.state, world)
    }
    fn make_dynamic(self) -> impl DynamicExecutable
    where
        'a: 'static,
    {
        self
    }
}
impl<'a, Args, State: 'a, E: ExecutableWithState<'a, State, Args>> TypeErasedExecutable<'a>
    for ExecutableWithStateStore<'a, State, Args, E>
{
    type Output = E::Output;

    fn execute(self, world: &World) -> Self::Output {
        self.executable.execute(self.state, world)
    }
    fn make_dynamic(self) -> impl DynamicExecutable
    where
        'a: 'static,
    {
        self
    }
}
impl<Args, State: 'static, E: ExecutableWithState<'static, State, Args>> DynamicExecutable
    for ExecutableWithStateStore<'static, State, Args, E>
{
    fn execute(self, world: &World) -> Box<dyn Any> {
        Box::new(self.executable.execute(self.state, world))
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

// TODO: Use the below macro to implement executable for functions, like so:
// impl_executable!(A ARef A ARef B BRef C CRef D DRef E ERef F FRef);
// This is currently held back by a compiler bug:
// https://github.com/rust-lang/rust/issues/100013
// I've subscribed to the issue and should get notified when it gets fixed. For
// now a temporary workaround is implemented below this macro below.
//
// TODO2: The codebase has changed since I originally wrote this, and this macro
// hasn't really been updated... need to update all the code under // Arguments
macro_rules! impl_executable {
    // No Arguments
    ($_unused:ident $_unused2:ident) => {
        impl<'a, Function, Output> Executable<'a, ()> for Function
        where
            Output: 'a,
            Function: FnOnce() -> Output + 'a
        {
            type Output = Output;

            fn execute(self, _: &World) -> Self::Output {
                self()
            }
        }
        impl<'a, Output, State: 'a, Func: 'a> ExecutableWithState<'a, State, ()> for Func
        where
            Output: 'a,
            Func: FnOnce(State) -> Output,
        {
            type Output = Output;

            fn with_state(self, state: State) -> impl Executable<'a, (), Output = Self::Output> {
                ExecutableWithStateStore {
                    executable: self,
                    state,
                    _ph: PhantomData
                }
            }
            #[allow(non_snake_case)]
            fn execute(self, state: State, _: &World) -> Self::Output {
                self(state)
            }
        }
    };

    // Arguments
    ($_unused:ident $_unused2:ident $($ty:ident $tyref:ident)*) => {
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
        impl<'a, Output, $($ty),*, $($tyref),*, Func: 'a> Executable<'a, ($($tyref,)*)> for Func
        where
            $($ty: ExecutableArg + 'a),*,
            $($tyref: ExecutableArgRef<EA = $ty> + 'a),*,
            Output: 'a,
            Func: FnOnce($($tyref),*) -> Output,
            Func: FnOnce($($tyref::Borrowed<'a, 'a>),*) -> Output
        {
            type Output = Output;

            #[allow(non_snake_case)]
            fn execute(self, world: &World) -> Output {
                let world_extended: &World = unsafe { &*(world as *const World) };

                $(let mut $ty = $ty::build(world_extended);)*
                let result = self($($tyref::borrow(unsafe { &mut *(&mut $ty as *mut $ty::Arg<'_>) })),*);
                $($ty.drop(world);)*

                result
            }
        }
        impl<'a, Output, State: 'a, $($ty),*, $($tyref),*, Func: 'a> ExecutableWithState<'a, State, ($($tyref,)*)> for Func
        where
            $($ty: ExecutableArg + 'a),*,
            $($tyref: ExecutableArgRef<EA = $ty> + 'a),*,
            Output: 'a,
            Func: FnOnce(State, $($tyref),*) -> Output,
            Func: FnOnce(State, $($tyref::Borrowed<'a, 'a>),*) -> Output
        {
            type Output = Output;

            fn with_state(self, state: State) -> impl Executable<'a, ($($tyref,)*), Output = Self::Output> {
                ExecutableWithStateStore {
                    executable: self,
                    state,
                    _ph: PhantomData
                }
            }
            #[allow(non_snake_case)]
            fn execute(self, state: State, world: &World) -> Self::Output {
                let world_extended: &World = unsafe { &*(world as *const World) };

                $(let mut $ty = $ty::build(world_extended);)*
                let result = self(state, $($tyref::borrow(unsafe { &mut *(&mut $ty as *mut $ty::Arg<'_>) })),*);
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

    fn accepts_executable<'a, Args: 'a>(func: impl Executable<'a, Args>) {
        let mut world = World::new();
        world.add_singleton(0_u32);
        world.add_singleton(1_i32);
        world.execute(func);
    }

    fn executable(_num: &mut Singleton<i32>) {}
    fn executable2(_num: &Singleton<i32>, _num2: &mut Singleton<u32>) {}
    fn executable3() {}

    fn stateful_executable(_state: u32) {}
    fn stateful_executable2(_state: u32, _num: &mut Singleton<i32>) {}

    #[test]
    fn type_test() {
        // Test functions
        accepts_executable(executable);
        accepts_executable(executable2);
        accepts_executable(executable3);

        // Test closures
        accepts_executable(|| {});
        accepts_executable(|_: &mut Singleton<i32>| {});
        accepts_executable(|_: &Singleton<i32>, _: &mut Singleton<i32>| {});

        // Test stateful functions
        accepts_executable(stateful_executable.with_state(0));
        accepts_executable(stateful_executable2.with_state(0));

        // Test closures that implement `FnOnce`
        let val = String::from("Hello!");
        accepts_executable(move || {
            println!("{val}");
            drop(val);
        });
        let val = String::from("Hello!");
        accepts_executable(move |_: &mut Singleton<i32>| {
            println!("{val}");
            drop(val);
        });
    }
}
