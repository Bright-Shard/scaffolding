use {
    crate::world::{ExecutableArg, World},
    alloc::boxed::Box,
    core::{any::Any, marker::PhantomData, ops::Fn},
};

/// Executables are functions that get data from the [`World`].
pub trait Executable: Clone + 'static {
    /// The return type.
    type Output: 'static;
    /// The executable's arguments.
    type Arguments: 'static;

    fn execute(&self, world: &mut World) -> Self::Output;
    // TODO: Replace with `impl FnOnce(&mut World) + Send + 'static` once
    // precise capturing is implemented:
    // https://github.com/rust-lang/rust/issues/123432
    fn execute_immut(&self, world: &World) -> (Self::Output, Box<dyn FnOnce(&mut World) + Send>);
}

/// A version of [`Executable`] that can be used as a trait object. Normal
/// executables require [`Clone`], which requires [`Sized`], and have
/// associated types for their output and mutations. This type has none of
/// those requirements, allowing it to be simply stored as a
/// `Box<dyn UnsizedExecutable>`. It's also automatically implemented for all
/// executables, which means any executable can be upcasted to a trait object
/// with this trait.
///
/// Because this trait doesn't have associated type, it relies heavily on trait
/// objects and dynamic typing, which adds overhead. Thus, [`Executable`]
/// should be preferred and used where possible.
pub trait UnsizedExecutable {
    /// The same as [`Executable::execute`], except this trait doesn't have an
    /// associated `Output` type, so the output has to be returned as an
    /// [`Any`] trait object.
    fn execute_unsized(&self, world: &mut World) -> Box<dyn Any>;

    fn execute_immut_unsized(
        &self,
        world: &World,
    ) -> (Box<dyn Any>, Box<dyn FnOnce(&mut World) + Send>);

    /// This replaces the [`Clone`] requirement in [`Executable`], which means
    /// this type doesn't have to be sized. Because this type is unsized, the
    /// cloned executable has to be returned as a trait object, so this method
    /// returns another [`UnsizedExecutable`].
    fn dyn_clone(&self) -> Box<dyn UnsizedExecutable>;
}
impl<E, Output> UnsizedExecutable for E
where
    Output: 'static,
    E: Executable<Output = Output>,
{
    fn execute_unsized(&self, world: &mut World) -> Box<dyn Any> {
        Box::new(self.execute(world))
    }
    fn execute_immut_unsized(
        &self,
        world: &World,
    ) -> (Box<dyn Any>, Box<dyn FnOnce(&mut World) + Send>) {
        let (output, apply) = self.execute_immut(world);

        (Box::new(output), Box::new(apply))
    }
    fn dyn_clone(&self) -> Box<dyn UnsizedExecutable> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn UnsizedExecutable> {
    fn clone(&self) -> Self {
        self.dyn_clone()
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
/// being dropped; after it's passed to the executable, it's used to build the
/// executable's mutations (see [`ExecutableArg::build_mutation`]), so it can't
/// be dropped until then.
///
/// This creates a new issue, however, because now executables can take both
/// mutable and immutable borrows, and those can be in any order - like these
/// two executables:
///
/// ```rust
/// use scaffolding::prelude::*;
///
/// fn executable(counter_state: &State<i32>, app: &mut Env<u32>) {
///   // Some very important code here
/// }
///
/// fn also_an_executable(app: &mut Env<u32>, counter_state: &State<i32>) {
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

pub trait IntoExecutable<Args>: Clone + 'static {
    type Output: 'static;
    type Executable: Executable<Output = Self::Output> + 'static;

    fn into_executable(self) -> Self::Executable;
}

#[repr(transparent)]
pub struct FunctionStore<Func: Clone, Args>(pub Func, pub PhantomData<Args>);
impl<Func: Clone, Args> Clone for FunctionStore<Func, Args> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<Args: 'static, F: Clone> IntoExecutable<Args> for FunctionStore<F, Args>
where
    Self: Executable + 'static,
{
    type Executable = Self;
    type Output = <Self as Executable>::Output;

    #[inline(always)]
    fn into_executable(self) -> Self::Executable {
        self
    }
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
        impl<Function, Output> IntoExecutable<()> for Function
        where
            Output: 'static,
            Function: 'static,
            Function: Fn() -> Output,
            Function: Clone
        {
            type Executable = FunctionStore<Self, ()>;
            type Output = Output;

            fn into_executable(self) -> Self::Executable {
                FunctionStore(self, PhantomData)
            }
        }

        impl<Function, Output> Executable for FunctionStore<Function, ()>
        where
            Output: 'static,
            Function: Fn() -> Output + Clone + 'static
        {
            type Output = Output;
            type Arguments = ();

            fn execute(&self, _: &mut World) -> Self::Output {
                self.0()
            }
            fn execute_immut(&self, _: &World) -> (Self::Output, Box<dyn FnOnce(&mut World) + Send>) {
                (self.0(), Box::new(|_| {}))
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
            Function: Clone
        {
            type Executable = FunctionStore<Self, ($($ty),*, $($tyref),*)>;
            type Output = Output;
            #[allow(unused_parens)]
            type Mutation = ($($ty::Mutation),*);

            fn into_executable(self) -> Self::Executable {
                FunctionStore(self, PhantomData)
            }
        }

        impl<Function, Output, $($ty),*, $($tyref),*> Executable for FunctionStore<Function, ($($ty),*, $($tyref),*)>
        where
            $(for<'a> $ty: ExecutableArg + 'a),*,
            $(for<'a> $tyref: ExecutableArgRef<EA = $ty> + 'a),*,
            Output: 'static,
            Function: 'static,
            Function: Fn($($tyref::Borrowed<'_, '_>),*) -> Output,
            Function: Fn($($tyref),*) -> Output,
            Function: Clone
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
        impl<Function, Output, $($ty),*, $($tyref),*> IntoExecutable<($($ty),*, $($tyref),*)> for Function
        where
            $($ty: ExecutableArg + 'static),*,
            $($tyref: ExecutableArgRef<EA = $ty> + 'static),*,
            Output: 'static,
            Function: 'static,
            Function: for<'a> Fn($($tyref::Borrowed<'a, 'a>),*) -> Output,
            Function: Fn($($tyref),*) -> Output,
            Function: Clone
        {
            type Executable = FunctionStore<Self, ($($ty),*, $($tyref),*)>;
            type Output = Output;

            fn into_executable(self) -> Self::Executable {
                FunctionStore(self, PhantomData)
            }
        }

        impl<Function, Output, $($ty),*, $($tyref),*> Executable for FunctionStore<Function, ($($ty),*, $($tyref),*)>
        where
            $($ty: ExecutableArg + 'static),*,
            $($tyref: ExecutableArgRef<EA = $ty> + 'static),*,
            Output: 'static,
            Function: 'static,
            Function: for<'a> Fn($($tyref::Borrowed<'a, 'a>),*) -> Output,
            Function: Fn($($tyref),*) -> Output,
            Function: Clone
        {
            type Output = Output;
            #[allow(unused_parens)]
            type Arguments = ($($ty),*);

            #[allow(non_snake_case)]
            fn execute(&self, world: &mut World) -> Self::Output {
                let world_extended: &World = unsafe { &*(world as *mut World) };

                $(let mut $ty = $ty::build(world_extended);)*
                let result = self.0($($tyref::borrow(unsafe { &mut *(&mut $ty as *mut $ty::Arg<'_>) })),*);
                $(($ty.on_drop())(world);)*

                result
            }
            #[allow(non_snake_case)]
            fn execute_immut(&self, world: &World) -> (Self::Output, Box<dyn FnOnce(&mut World) + Send>) {
                let world_extended: &World = unsafe { &*(world as *const World) };

                $(let mut $ty = $ty::build(world_extended);)*

                let result = self.0($($tyref::borrow(unsafe { &mut *(&mut $ty as *mut $ty::Arg<'_>) })),*);

                $(let $ty = $ty.on_drop();)*

                (result, Box::new(move |world| { $( ($ty)(world); )* }))
            }
        }

        impl_executable_workaround!($($ty $tyref)*);
    };
}
impl_executable_workaround!(A ARef A ARef B BRef C CRef D DRef E ERef F FRef);

#[cfg(test)]
mod tests {
    use {super::*, crate::prelude::*};

    fn accepts_executable<Args>(func: impl IntoExecutable<Args>) {
        let mut world = World::new();
        world.add_singleton(0_u32);
        world.add_singleton(1_i32);
        world.execute(func);
    }

    fn executable(_num: &mut Singleton<i32>) {}
    fn executable2(_num: &Singleton<i32>, _num2: &mut Singleton<u32>) {}
    fn executable3() {}

    #[test]
    fn type_test() {
        accepts_executable(executable);
        accepts_executable(executable2);
        accepts_executable(executable3);
        accepts_executable(|| {});
        accepts_executable(|_: &mut Singleton<i32>| {});
        accepts_executable(|_: &Singleton<i32>, _: &mut Singleton<i32>| {});
    }
}
