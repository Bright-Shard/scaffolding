use {
    super::{unix_common, OsTrait, OsType},
    core::{alloc::Layout, ffi::c_void, ptr::NonNull},
};

pub struct Os;

impl OsTrait for Os {
    const TYPE: OsType = OsType::Linux;

    fn page_size() -> usize {
        unix_common::page_size()
    }

    fn reserve(amount: usize) -> Option<NonNull<c_void>> {
        unix_common::reserve(amount)
    }
    unsafe fn commit(ptr: NonNull<c_void>, amount: usize) {
        unix_common::commit(ptr, amount)
    }
    fn allocate(layout: Layout) -> Option<NonNull<c_void>> {
        unix_common::allocate(layout)
    }

    unsafe fn dereserve(ptr: NonNull<c_void>, amount: usize) {
        unix_common::dereserve(ptr, amount)
    }
    unsafe fn decommit(ptr: NonNull<c_void>, amount: usize) {
        unix_common::decommit(ptr, amount)
    }
    unsafe fn deallocate(ptr: NonNull<c_void>, amount: usize) {
        unix_common::deallocate(ptr, amount)
    }

    fn current_thread() -> i64 {}
}
