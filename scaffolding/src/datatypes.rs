//! Custom data structures that simplify the borrow checker and other tasks.

pub mod arenavec;
pub mod stackvec;
pub mod typemap;
pub mod warehouse;

#[doc(inline)]
pub use {arenavec::ArenaVec, stackvec::StackVec, typemap::TypeMap, warehouse::Warehouse};
