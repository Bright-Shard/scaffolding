use {
    crate::display::platform::PlatformTrait,
    std::{
        env,
        os::{fd::FromRawFd, unix::net::UnixStream},
        path::PathBuf,
    },
};

pub struct WaylandPlatform {}
impl PlatformTrait for WaylandPlatform {
    fn init() -> Option<Self> {
        // Try to locate the wayland display from the `WAYLAND_SOCKET` variable
        let mut compositor = if let Ok(socket) = env::var("WAYLAND_SOCKET") {
            if let Ok(socket) = socket.parse::<i32>() {
                Some(unsafe { UnixStream::from_raw_fd(socket) })
            } else {
                None
            }
        } else {
            None
        };

        // Fall back to using the `WAYLAND_DISPLAY` variable
        if compositor.is_none() {
            if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
                if let Ok(display) = env::var("WAYLAND_DISPLAY") {
                    let path = PathBuf::from(runtime_dir);
                    compositor = UnixStream::connect(path.join(display)).ok();
                } else {
                    // If all else fails just try to connect to `wayland-0`
                    compositor = UnixStream::connect(runtime_dir + "wayland-0").ok();
                }
            }
        }

        None
    }
}
