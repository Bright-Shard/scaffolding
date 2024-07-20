use {super::PlatformTrait, scaffolding::utils::ScaffoldingBox, wayland::WaylandPlatform};

mod socket;
mod wayland;

// TODO: Benchmark if an enum or trait object would perform better here
/// On Linux, the platform could be either X11 or Wayland. This wrapper type
/// will store one of either.
#[repr(transparent)]
pub struct Platform(ScaffoldingBox<dyn LinuxPlatform>);
impl PlatformTrait for Platform {
    fn init() -> Option<Self> {
        let boxed = ScaffoldingBox::new(WaylandPlatform::init()?);
        let dyn_box = unsafe { ScaffoldingBox::from_raw(boxed.as_raw() as _) };
        Some(Self(dyn_box))
    }
}

trait LinuxPlatform {}
impl<T: PlatformTrait> LinuxPlatform for T {}
