use {
    super::{OsTrait, OsType},
    core::{
        alloc::Layout,
        ffi::c_void,
        ptr::{self, NonNull},
    },
    libc::{
        free, mmap, mprotect, munmap, posix_memalign, sysconf, MAP_ANONYMOUS, MAP_FAILED,
        MAP_PRIVATE, PROT_NONE, PROT_READ, PROT_WRITE, _SC_PAGE_SIZE,
    },
};

pub struct Os;

// Unix implementation is largely taken from Speykious' csussus project:
// https://github.com/Speykious/csussus/blob/main/src/arena.rs

impl OsTrait for Os {
    #[cfg(target_os = "linux")]
    const TYPE: OsType = OsType::Linux;
    #[cfg(target_os = "macos")]
    const TYPE: OsType = OsType::MacOS;

    fn page_size() -> usize {
        unsafe { sysconf(_SC_PAGE_SIZE) as usize }
    }

    fn reserve(amount: usize) -> Option<NonNull<c_void>> {
        let ptr = unsafe {
            mmap(
                ptr::null_mut(),
                amount,
                PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if ptr == MAP_FAILED {
            return None;
        }

        NonNull::new(ptr)
    }
    unsafe fn commit(ptr: NonNull<c_void>, amount: usize) {
        unsafe {
            mprotect(ptr.as_ptr(), amount, PROT_READ | PROT_WRITE);
        }
    }
    fn allocate(layout: Layout) -> Option<NonNull<c_void>> {
        let mut ptr = ptr::null_mut();
        unsafe {
            posix_memalign(&mut ptr as *mut *mut c_void, layout.align(), layout.size());
        }
        NonNull::new(ptr)
    }

    unsafe fn dereserve(ptr: NonNull<c_void>, amount: usize) {
        munmap(ptr.as_ptr(), amount);
    }
    unsafe fn decommit(ptr: NonNull<c_void>, amount: usize) {
        mprotect(ptr.as_ptr(), amount, PROT_NONE);
    }
    unsafe fn deallocate(ptr: NonNull<c_void>, _: usize) {
        unsafe { free(ptr.as_ptr()) };
    }
}
