//! Multithreaded utilities for Scaffolding.

use {
    crate::{
        utils::AssumeSyncSend,
        world::{Executable, IntoExecutable, World},
    },
    std::thread,
};

/// The `execute_in_parallel` method in the [`World`]. This is a separate trait
/// so that the method can be implemented multiple times, for different numbers
/// of executables. For example, this is implemented for one executable, but
/// also tuples of two or more.
pub trait ExecuteInParallel<Args, E, O> {
    /// Runs a tuple of executables in parallel. All of the executables will be
    /// run at the same time, and then any changes they make to the [`World`]
    /// will be applied after they all finish running.
    fn execute_in_parallel(&mut self, executables: E) -> O;
}

impl<Args, E: IntoExecutable<'static, Args>> ExecuteInParallel<Args, E, E::Output> for World {
    fn execute_in_parallel(&mut self, executables: E) -> E::Output {
        let out = executables.into_executable().execute(self);
        self.apply_msgs();
        out
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
            ($($generic::Output),*),
        >

        for World
        where
            $(
                $generic: IntoExecutable<'static, $args> + Send,
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
                        let world = unsafe { AssumeSyncSend::new(self as *const World) };

                        thread::spawn(move || {
                            let world = unsafe { &*world.take() };
                            let executable = tuple_idx!(executables, $generic);
                            let executable = executable.into_executable();
                            let output = executable.execute(world);
                            output
                        })
                    };
                )*
                $(
                    let $generic = $generic.join().unwrap();
                )*
                self.apply_msgs();
                ($($generic),*)
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
        world
            .add_singleton(thread::current().id())
            .add_singleton(NumThreads(0))
            .add_msg_handler(|world, _: Msg<MsgNewThread>| {
                let num_threads: &mut NumThreads = world.get_singleton_mut();
                num_threads.0 += 1;
            });

        world.execute_in_parallel((
            parallel_executable,
            parallel_executable,
            parallel_executable,
            parallel_executable,
        ));

        let num_threads: &NumThreads = world.get_singleton();
        assert_eq!(num_threads.0, 4);
    }

    fn parallel_executable(
        main_thread_id: &Singleton<ThreadId>,
        num_threads: &Singleton<NumThreads>,
        msg_sender: &MsgSender,
    ) {
        let current_thread_id = thread::current().id();
        println!("Thread {current_thread_id:?} running.");
        assert_ne!(**main_thread_id, current_thread_id);
        assert_eq!(num_threads.0, 0);

        msg_sender.send(MsgNewThread);
    }

    struct NumThreads(u8);
    struct MsgNewThread;
}
