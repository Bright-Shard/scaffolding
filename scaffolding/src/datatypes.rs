//! Custom data structures that simplify the borrow checker and other tasks.

pub mod arenavec;
pub mod stackvec;
pub mod typemap;
pub mod uniq;
pub mod warehouse;

#[doc(inline)]
pub use {
    arenavec::ArenaVec,
    stackvec::StackVec,
    typemap::TypeMap,
    uniq::{uniq_key, Uniq},
    warehouse::Warehouse,
};
