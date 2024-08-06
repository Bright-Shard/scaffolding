//! A custom socket implementation that supports sending ancillary data. The
//! standard library's implementation also supports sending ancillary data over
//! sockets, but it is currently unstable, and thus not useable in stable rust.
//!
//! Ancillary data is extra, operating system data that's sent over a socket.
//! Wayland uses ancillary data to send file descriptors to its compositor.

use std::{
    io::IoSlice,
    mem,
    os::{
        fd::{AsRawFd, RawFd},
        unix::net::UnixStream,
    },
    ptr::{self, NonNull},
};

pub fn send(socket: &UnixStream, data: &[u8], ancillary_fd: Option<RawFd>) {
    let socket_fd = socket.as_raw_fd();

    let mut data = IoSlice::new(data);
    // Safety: The [`IoSlice`] docs state [`IoSlice`] is ABI-compatible with
    // [`IOVec`].
    let iov = &mut data as *mut IoSlice as *mut IOVec;

    let mut control = ancillary_fd.map(|fd| ControlMessageHeader {
        len: mem::size_of::<ControlMessageHeader<4>>(),
        level: ffi::SOL_SOCKET,
        ty: ffi::SCM_RIGHTS,
        data: fd.to_ne_bytes(),
    });
    let (control, control_len) = if let Some(ref mut control) = control {
        (
            control as *mut ControlMessageHeader<4> as *mut (),
            mem::size_of_val(control),
        )
    } else {
        (ptr::null_mut(), 0)
    };

    let msg = MessageHeader {
        name: ptr::null_mut(),
        name_len: 0,
        iov,
        iov_len: 1,
        control,
        control_len,
        msg_flags: 0,
    };

    unsafe {
        ffi::sendmsg(socket_fd, &msg as *const MessageHeader, 0);
    }
}

/// The header for data sent between unix sockets.
///
/// See `rcvmsg(2)`: https://man7.org/linux/man-pages/man2/recvmsg.2.html.
#[repr(C)]
pub struct MessageHeader {
    pub name: *mut (),
    pub name_len: u32,
    /// A pointer to a vector of [`IOVec`]s.
    pub iov: *mut IOVec,
    /// The number of [`IOVec`]s in `self.iov`.
    pub iov_len: usize,
    /// A pointer to a buffer containing [`ControlMessageHeader`]s.
    pub control: *mut (),
    /// The size of `self.control`.
    pub control_len: usize,
    /// This field is only used while reading messages from a socket, so we ignore it.
    pub msg_flags: i32,
}

#[repr(C)]
pub struct ControlMessageHeader<const DATA_LEN: usize> {
    pub len: usize,
    pub level: i32,
    pub ty: i32,
    pub data: [u8; DATA_LEN],
}

/// A simple vector type. Rust's [`IoSlice`] is ABI-compatible with this.
///
/// aka: awww poor little baby C doesn't have a built-in vec type :c
#[repr(C)]
pub struct IOVec {
    pub base: NonNull<()>,
    pub len: usize,
}

mod ffi {
    use super::*;

    pub const SOL_SOCKET: i32 = 1;
    pub const SCM_RIGHTS: i32 = 1;

    extern "C" {
        /// The syscall that sends a message to a socket.
        pub fn sendmsg(fd: RawFd, msg: *const MessageHeader, flags: i32) -> isize;
    }
}
