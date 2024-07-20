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

impl World {
    pub fn execute_in_parallel(&self) {}
}
