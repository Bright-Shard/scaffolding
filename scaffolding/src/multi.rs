//! Multithreaded data types and methods for Scaffolding.

use {
    crate::{
        datatypes::TypeMap,
        utils::AssumeSyncSend,
        world::{Executable, IntoExecutable, Mutation, World},
    },
    std::thread,
};

pub trait ExecuteInParallel<Args, E, M, O> {
    fn execute_in_parallel(&mut self, executables: E) -> O;
    fn execute_in_parallel_delayed_mutation(&self, executables: E) -> (O, M);
}
impl<Args, E: IntoExecutable<Args>> ExecuteInParallel<Args, E, E::Mutation, E::Output> for World {
    fn execute_in_parallel(&mut self, executables: E) -> E::Output {
        executables
            .into_executable()
            .execute(self, &TypeMap::default())
    }
    fn execute_in_parallel_delayed_mutation(&self, executables: E) -> (E::Output, E::Mutation) {
        executables
            .into_executable()
            .execute_delayed_mutation(self, &TypeMap::default())
    }
}

macro_rules! tuple_idx {
    ($self:ident, A) => {
        $self.0
    };
    ($self:ident, B) => {
        $self.1
    };
    ($self:ident, C) => {
        $self.2
    };
    ($self:ident, D) => {
        $self.3
    };
    ($self:ident, E) => {
        $self.4
    };
    ($self:ident, F) => {
        $self.5
    };
}
macro_rules! impl_execute_in_parallel {
    ($($generic:ident $args:ident)*) => {
        #[allow(non_snake_case)]
        impl<$($args, $generic),*>

        ExecuteInParallel<
            ($($args),*),
            ($($generic),*),
            ($($generic::Mutation),*),
            ($($generic::Output),*),
        >

        for World
        where
            $(
                $generic: IntoExecutable<$args> + Send,
                $generic::Mutation: Send,
                $generic::Output: Send
            ),*

        {
            fn execute_in_parallel
                (&mut self, executables: ($($generic),*))
                -> ($($generic::Output),*)
            {
                $(
                    let $generic = {
                        // `thread::spawn` requires a 'static lifetime, so we
                        // can't use &self here, because we'd have to borrow
                        // self for 'static
                        //
                        // this is still safe because the world is only needed
                        // until the thread finishes running... which happens
                        // in this very method
                        let world = unsafe { AssumeSyncSend::new(&*self as *const World) };

                        thread::spawn(move || {
                            let world = unsafe { &*world.take() };
                            let executable = tuple_idx!(executables, $generic);
                            let output = world.execute_delayed_mutation(executable, None);
                            output
                        })
                    };
                )*
                $(
                    let ($generic, $args) = $generic.join().unwrap();
                )*
                $(
                    $args.apply(self);
                )*
                ($($generic),*)
            }
            fn execute_in_parallel_delayed_mutation
                (&self, executables: ($($generic),*))
                -> (($($generic::Output),*), ($($generic::Mutation),*))
            {
                $(
                    let $generic = {
                        // `thread::spawn` requires a 'static lifetime, so we
                        // can't use &self here, because we'd have to borrow
                        // self for 'static
                        //
                        // this is still safe because the world is only needed
                        // until the thread finishes running... which happens
                        // in this very method
                        let world = unsafe { AssumeSyncSend::new(&*self as *const World) };

                        thread::spawn(move || {
                            let world = unsafe { &*world.take() };
                            let executable = tuple_idx!(executables, $generic);
                            let output = world.execute_delayed_mutation(executable, None);
                            output
                        })
                    };
                )*
                $(
                    let ($generic, $args) = $generic.join().unwrap();
                )*
                (($($generic),*), ($($args),*))
            }
        }
    };
}
impl_execute_in_parallel!(A AArgs B BArgs);
impl_execute_in_parallel!(A AArgs B BArgs C CArgs);
impl_execute_in_parallel!(A AArgs B BArgs C CArgs D DArgs);
impl_execute_in_parallel!(A AArgs B BArgs C CArgs D DArgs E EArgs);

#[cfg(test)]
mod tests {
    use {
        crate::plugin_prelude::*,
        std::thread::{self, ThreadId},
    };

    /// We create a world with 2 states: the `ThreadId` of the program's main
    /// thread, and a `u32` starting at 0.
    /// We then use `World::execute_in_parallel` to spawn a bunch of threads.
    /// Each one checks that their thread ID is different from the `ThreadId`
    /// in the world, to make sure it's actually running on a different thread.
    /// They all then increment the `u32` by 1.
    /// At the end we assert the `u32` is equal to the number of threads we
    /// spawned.
    #[test]
    fn test_parallel() {
        let mut world = World::new();
        world.add_state(thread::current().id()).add_state(0u32);

        world.execute_in_parallel((
            parallel_executable,
            parallel_executable,
            parallel_executable,
            parallel_executable,
        ));
        let thread_count: u32 = *world.get_state();
        assert_eq!(thread_count, 4);
    }

    fn parallel_executable(main_thread_id: &State<ThreadId>, thread_counter: &mut ThreadCounter) {
        let current_thread_id = thread::current().id();
        println!("Thread {current_thread_id:?} running.");
        assert_ne!(**main_thread_id, current_thread_id);
        assert_eq!(thread_counter.count, 0);
        thread_counter.increment = true;
    }

    #[derive(Clone)]
    struct ThreadCounter {
        pub count: u32,
        pub increment: bool,
    }
    impl ExecutableArg for ThreadCounter {
        type Arg<'a> = Self;
        type Mutation = Self;

        fn from_world_and_env<'a>(world: &'a ImmutableWorld, _: &'a TypeMap) -> Self::Arg<'a> {
            Self {
                count: *world.get_state(),
                increment: false,
            }
        }
        fn build_mutation(self) -> Self::Mutation {
            self
        }
    }
    impl Mutation for ThreadCounter {
        type Reverse = ();

        fn apply(self, world: &mut World) {
            if self.increment {
                *world.get_state_mut::<u32>() += 1;
            }
        }
        fn build_reverse(&self, _: &World) -> Self::Reverse {}
    }
}
