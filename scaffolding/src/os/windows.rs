use {
    super::{OsTrait, OsType},
    crate::utils::bitflags,
    core::{
        alloc::Layout,
        ffi::c_void,
        mem::MaybeUninit,
        ptr::{self, NonNull},
    },
};

pub struct Os;

impl OsTrait for Os {
    const TYPE: OsType = OsType::Windows;

    fn page_size() -> usize {
        let mut sysinfo = MaybeUninit::uninit();
        unsafe {
            GetSystemInfo(sysinfo.as_mut_ptr());
        }
        unsafe { sysinfo.assume_init() }.page_size as usize
    }

    fn reserve(amount: usize) -> Option<NonNull<c_void>> {
        NonNull::new(unsafe {
            VirtualAlloc(
                ptr::null_mut(),
                amount,
                AllocationType::Reserve.into(),
                MemoryProtection::ReadWrite.into(),
            )
        })
    }
    unsafe fn commit(ptr: NonNull<c_void>, amount: usize) {
        unsafe {
            VirtualAlloc(
                ptr.as_ptr(),
                amount,
                AllocationType::Commit.into(),
                MemoryProtection::ReadWrite.into(),
            );
        }
    }
    fn allocate(layout: Layout) -> Option<NonNull<c_void>> {
        NonNull::new(unsafe {
            VirtualAlloc(
                ptr::null_mut(),
                layout.size(),
                AllocationType::Reserve | AllocationType::Commit,
                MemoryProtection::ReadWrite.into(),
            )
        })
    }

    unsafe fn dereserve(ptr: NonNull<c_void>, amount: usize) {
        unsafe {
            VirtualFree(ptr.as_ptr(), amount, FreeType::Release.into());
        }
    }
    unsafe fn decommit(ptr: NonNull<c_void>, amount: usize) {
        unsafe {
            VirtualFree(ptr.as_ptr(), amount, FreeType::Decommit.into());
        }
    }
    unsafe fn deallocate(ptr: NonNull<c_void>, amount: usize) {
        unsafe {
            VirtualFree(ptr.as_ptr(), amount, FreeType::Decommit | FreeType::Release);
        }
    }
}

// Win32 API Types:
// SHORT: i16
// WORD: u16
// DWORD: u32
// LPVOID: *mut c_void
// DWORD_PTR: usize

#[repr(C)]
#[derive(Clone, Copy)]
struct SystemInfoProcessor {
    pub processor_architecture: u16,
    pub reserved: u16,
}
#[repr(C)]
union SystemInfoUnion {
    pub oem_id: u32,
    pub system_info: SystemInfoProcessor,
}

/// https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/ns-sysinfoapi-system_info
#[repr(C)]
struct SystemInfo {
    pub oem: SystemInfoUnion,
    pub page_size: u32,
    pub minimum_application_address: *mut c_void,
    pub maximum_application_address: *mut c_void,
    pub active_processor_mask: usize,
    pub number_of_processors: u32,
    pub processor_type: u32,
    pub allocation_granularity: u32,
    pub processor_level: u16,
    pub processor_revision: u16,
}

bitflags! {
    struct AllocationTypes: u32;
    bitflags AllocationType {
        Commit = 0x00001000,
        Reserve = 0x00002000,
    }
}
bitflags! {
    struct MemoryProtectionType: u32;
    bitflags MemoryProtection {
        // Execute = 0x10,
        // ExecuteRead = 0x20,
        // ExecuteReadWrite = 0x40,
        // ReadyOnly = 0x02,
        ReadWrite = 0x04
    }
}
bitflags! {
    struct FreeTypes: u32;
    bitflags FreeType {
        Decommit = 0x00004000,
        Release = 0x00008000,
    }
}

#[link(name = "kernel32")]
extern "C" {
    fn GetSystemInfo(lpSystemInfo: *mut SystemInfo);
    fn VirtualAlloc(
        lpAddress: *mut c_void,
        dwSize: usize,
        flAllocationType: AllocationTypes,
        flProtect: MemoryProtectionType,
    ) -> *mut c_void;
    fn VirtualFree(lpAddress: *mut c_void, dwSize: usize, dwFreeType: FreeTypes) -> bool;
}
