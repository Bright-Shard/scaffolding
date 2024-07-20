use {
    crate::{datatypes::StackVec, world::World},
    alloc::{boxed::Box, vec::Vec},
};

/// A type that mutates the world.
pub trait Mutation: Clone + 'static {
    type Reverse: Mutation;

    fn apply(self, world: &mut World);
    fn build_reverse(&self, world: &World) -> Self::Reverse;
}
impl Mutation for () {
    type Reverse = ();

    #[inline(always)]
    fn apply(self, _: &mut World) {}
    #[inline(always)]
    fn build_reverse(&self, _: &World) -> Self::Reverse {}
}
impl<M: Mutation> Mutation for Vec<M> {
    type Reverse = Vec<M::Reverse>;

    fn apply(self, world: &mut World) {
        self.into_iter().for_each(|mutation| {
            mutation.apply(world);
        });
    }
    fn build_reverse(&self, world: &World) -> Self::Reverse {
        self.iter()
            .map(|mutation| mutation.build_reverse(world))
            .collect()
    }
}
impl<M: Mutation, const SIZE: usize> Mutation for StackVec<M, SIZE> {
    type Reverse = StackVec<M::Reverse, SIZE>;

    fn apply(self, world: &mut World) {
        self.into_iter().for_each(|mutation| {
            mutation.apply(world);
        });
    }
    fn build_reverse(&self, world: &World) -> Self::Reverse {
        self.iter()
            .map(|mutation| mutation.build_reverse(world))
            .collect()
    }
}

/// An unsized version of [`Mutation`]. This trait is implemented automatically
/// for all types that implement [`Mutation`]. Unlike [`Mutation`], this type
/// can be used as a trait object.
pub trait UnsizedMutation {
    fn apply_unsized(self: Box<Self>, world: &mut World);
    fn build_reverse_unsized(&self, world: &World) -> Box<dyn UnsizedMutation>;
    fn dyn_clone(&self) -> Box<dyn UnsizedMutation>;
}
impl<M: Mutation> UnsizedMutation for M {
    fn apply_unsized(self: Box<Self>, world: &mut World) {
        <Self as Mutation>::apply(*self, world)
    }
    fn build_reverse_unsized(&self, world: &World) -> Box<dyn UnsizedMutation> {
        Box::new(self.build_reverse(world))
    }
    fn dyn_clone(&self) -> Box<dyn UnsizedMutation> {
        Box::new(self.clone())
    }
}
impl Mutation for Box<dyn UnsizedMutation> {
    type Reverse = Box<dyn UnsizedMutation>;

    fn apply(self, world: &mut World) {
        self.apply_unsized(world)
    }
    fn build_reverse(&self, world: &World) -> Self::Reverse {
        self.build_reverse_unsized(world)
    }
}
impl Clone for Box<dyn UnsizedMutation> {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

#[derive(Clone)]
pub struct MutationSet {
    mutations: Vec<Box<dyn UnsizedMutation>>,
}
impl MutationSet {
    pub fn new(mutations: Vec<Box<dyn UnsizedMutation>>) -> Self {
        Self { mutations }
    }
}
impl Mutation for MutationSet {
    type Reverse = Self;

    fn apply(self, world: &mut World) {
        for mutation in self.mutations {
            mutation.apply_unsized(world);
        }
    }
    fn build_reverse(&self, world: &World) -> Self::Reverse {
        Self {
            mutations: self
                .mutations
                .iter()
                .map(|mutation| mutation.build_reverse_unsized(world))
                .collect(),
        }
    }
}

macro_rules! id {
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
macro_rules! mutation_tuple_impl {
    ($($generic:tt)*) => {
        impl<$($generic: Mutation),*> Mutation for ($($generic,)*) {
            type Reverse = ($($generic::Reverse,)*);

            fn apply(self, world: &mut World) {
                $(id!(self, $generic).apply(world);)*
            }

            fn build_reverse(&self, world: &World) -> Self::Reverse {
                ($(id!(self, $generic).build_reverse(world),)*)
            }
        }
    };
}

mutation_tuple_impl!(A);
mutation_tuple_impl!(A B);
mutation_tuple_impl!(A B C);
mutation_tuple_impl!(A B C D);
mutation_tuple_impl!(A B C D E);
mutation_tuple_impl!(A B C D E F);
