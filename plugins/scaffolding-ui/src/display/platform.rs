//! A "platform" is a set of low-level APIs for creating GUI apps. For example,
//! on macOS, the platform is AppKit; on Windows, it's the Win32 API; and on
//! Linux, it's either Wayland or X11.
//!
//! This module is similar to the `os` module in Scaffolding. It's the set of
//! functions ScaffoldingUI needs access to in order to create a GUI app.

use scaffolding::world::World;

/// Platform APIs that ScaffoldingUI needs access to.
pub trait PlatformTrait: Sized {
    fn new(world: &mut World) -> Option<Self>;
}

// Platform implementations

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::Platform;
