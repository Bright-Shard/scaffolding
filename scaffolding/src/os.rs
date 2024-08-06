//! Low-level OS APIs used by Scaffolding.
//!
//! The API used by Scaffolding is described in [`OsTrait`]. Each OS'
//! implementation is in its own file.

use core::{alloc::Layout, ffi::c_void, ptr::NonNull};

/// OS functions Scaffolding needs access to.
pub trait OsTrait {
    /// Which OS this program is running on.
    const TYPE: OsType;

    /// The size of a single memory page in this OS.
    fn page_size() -> usize;

    /// Reserve `amount` bytes of virtual memory. This shouldn't allocate
    /// an memory, but instead just reserve virtual addresses to be
    /// allocated later with [`OsTrait::commit`].
    ///
    /// Note that, unlike [`OsTrait::allocate`], the reserved memory may not be
    /// properly aligned for a specific type. You are responsible for alignment.
    fn reserve(amount: usize) -> Option<NonNull<c_void>>;
    /// Commit `amount` bytes of reserved memory at `ptr`.
    ///
    /// # Safety
    /// `ptr` must point to a valid region of memory that was reserved with
    /// [`OsTrait::reserve`].
    unsafe fn commit(ptr: NonNull<c_void>, amount: usize);
    /// Allocate memory for the given layout.
    fn allocate(layout: Layout) -> Option<NonNull<c_void>>;

    /// Release memory reserved with [`OsTrait::reserve`].
    ///
    /// # Safety
    /// `ptr` must point to a valid region of memory that was reserved
    /// with [`OsTrait::reserve`]. That memory should not be committed.
    unsafe fn dereserve(ptr: NonNull<c_void>, amount: usize);
    /// Release memory committed with [`OsTrait::commit`].
    ///
    /// # Safety
    /// `ptr` must point to a valid region of memory that was reserved
    /// with [`OsTrait::reserve`] and then committed with
    /// [`OsTrait::commit`].
    unsafe fn decommit(ptr: NonNull<c_void>, amount: usize);
    /// Release memory allocated with [`OsTrait::allocate`].
    ///
    /// # Safety
    /// `ptr` must point to a valid region of memory that was allocated
    /// with [`OsTrait::allocate`]. That memory shouldn't be reserved
    /// or committed.
    unsafe fn deallocate(ptr: NonNull<c_void>, amount: usize);
}

/// Miscellaneous OS information. An instance of this is stored as a global,
/// which you can get with [`OsMetadata::global()`] or
/// [`OsMetadata::global_unchecked()`].
pub struct OsMetadata {
    /// The size of a single memory page on this OS.
    pub page_size: usize,
}
impl OsMetadata {
    /// Which OS this program is running on.
    pub const TYPE: OsType = Os::TYPE;

    /// Align a number to the OS' page size.
    #[inline(always)]
    pub fn page_align(&self, num: usize) -> usize {
        utils::align(num, self.page_size)
    }
}
impl Default for OsMetadata {
    fn default() -> Self {
        Self {
            page_size: Os::page_size(),
        }
    }
}

/// A list of operating systems supported by Scaffolding. The current operating
/// system is stored in [`Os::TYPE`] and [`OsMetadata::TYPE`].
pub enum OsType {
    Linux,
    MacOS,
}

// OS implementations

#[cfg(target_family = "unix")]
mod unix_common;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as os_impl;
#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
use mac as os_impl;

#[doc(inline)]
/// The OS implementation of [`OsTrait`].
pub use os_impl::Os;

use crate::utils;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("Scaffolding isn't currently supported for the operating system you're building for. Feel free to comment on or open an issue on GitHub.");

/// A basic global allocator using the OS' allocate and deallocate functions.
///
/// This just calls the [`OsTrait::allocate`] and
/// [`OsTrait::deallocate`] functions.
/// It doesn't have good functions for resizing allocations or
/// allocating with 0s. The standard library will probably have a better
/// global allocator than this one.
#[cfg(feature = "os-allocator")]
mod os_allocator {
    use {
        super::{Os, OsTrait},
        alloc::alloc::GlobalAlloc,
        core::{alloc::Layout, ptr::NonNull},
    };

    unsafe impl GlobalAlloc for Os {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            Os::allocate(layout).unwrap().as_ptr().cast()
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            Os::deallocate(NonNull::new(ptr).unwrap().cast(), layout.size());
        }
    }
}
