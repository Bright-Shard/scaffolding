use {super::PlatformTrait, scaffolding::prelude::World, wayland::WaylandPlatform};

mod socket;
mod wayland;

/// On Linux, the platform could be either X11 or Wayland. Scaffolding will try
/// to connect to a Wayland server, and then fall back on X11 if Wayland fails.
/// This wrapper type handles that and also forwards all platform calls to the
/// correct platform (X11/Wayland) as needed.
#[repr(transparent)]
pub struct Platform(LinuxPlatform);
impl PlatformTrait for Platform {
    fn new(world: &mut World) -> Option<Self> {
        match WaylandPlatform::new(world) {
            Some(platform) => {
                world.add_singleton(platform);
                Some(Self(LinuxPlatform::Wayland))
            }
            None => todo!("Fallback to X11"),
        }
    }
}

enum LinuxPlatform {
    Wayland,
    X11,
}
