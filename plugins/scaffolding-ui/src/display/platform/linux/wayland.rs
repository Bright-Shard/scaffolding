use {
    crate::display::platform::PlatformTrait,
    core::panic,
    scaffolding::{datatypes::ArenaVec, world::World},
    std::{
        env,
        os::{fd::FromRawFd, unix::net::UnixStream},
        path::PathBuf,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread,
        time::Duration,
    },
    wire::*,
};

mod events;
mod wire;

type WaylandCallback = Box<dyn FnOnce(&WaylandPlatform, u32)>;

/// Wayland's protocol contains many global singletons. This struct stores all
/// of those singletons.
struct WaylandGlobals {
    display: Object<Display>,
    registry: Object<Registry>,
    compositor: Object<Compositor>,
    xdg_wm_base: Object<XdgWmBase>,
    shm: Object<Shm>,
}

pub struct WaylandPlatform {
    compositor: UnixStream,
    objects: Vec<Option<SomeObject>>,
    globals: WaylandGlobals,
    callbacks: ArenaVec<(u32, WaylandCallback)>,
    /// Used to store messages to read from the compositor.
    read_buffer: ArenaVec<u8>,
    /// Used to store messages to send to the compositor.
    write_buffer: ArenaVec<u8>,
}
impl PlatformTrait for WaylandPlatform {
    fn new(_: &mut World) -> Option<Self> {
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
        let compositor = compositor?;

        let display = Object::with_id(1);

        let globals = WaylandGlobals {
            display,
            registry: Object::with_id(2),
            xdg_wm_base: Object::with_id(0),
            shm: Object::with_id(0),
            compositor: Object::with_id(0),
        };

        let mut objects = Vec::with_capacity(20);
        objects.push(None);
        objects.push(Some(SomeObject::Display(Object::with_id(1))));
        objects.push(Some(SomeObject::Registry(Object::with_id(2))));

        let mut this = Self {
            compositor,
            objects,
            globals,
            callbacks: ArenaVec::default(),
            // Wire messages store their length as u16, so a u16 is the max
            // either of these can possibly be
            read_buffer: ArenaVec::with_capacity(u16::MAX as usize),
            write_buffer: ArenaVec::with_capacity(u16::MAX as usize),
        };

        let display = this.globals.display;
        display.get_registry(&mut this, NewId::with_id(2));

        this.sync();

        // Globals to make sure we received in the last sync
        let globals_to_check = [this.globals.xdg_wm_base.id, this.globals.shm.id];
        for global in globals_to_check {
            if global == 0 {
                panic!(
                    "Scaffolding failed to get a Wayland global that it needs to work correctly."
                );
            }
        }

        Some(this)
    }
}
impl WaylandPlatform {
    /// Block until the compositor processes all incoming events. This will
    /// process events from the compositor while blocking.
    pub fn sync(&mut self) {
        let synced = Arc::new(AtomicBool::new(false));
        let callback_synced = synced.clone();

        let callback = self.create_callback(Box::new(move |_, _| {
            callback_synced.store(true, Ordering::Release);
        }));

        let display = self.globals.display;
        display.sync(self, callback);

        loop {
            // println!("screee");
            thread::sleep(Duration::from_millis(1));

            self.process_next_event(false);

            if synced.load(Ordering::Relaxed) {
                break;
            }
        }
    }

    /// Try to reuse an object ID we don't need, or otherwise make a new one.
    pub fn new_id<I: Interface>(&mut self) -> NewId<I> {
        for (idx, obj) in self.objects[1..].iter().enumerate() {
            if obj.is_none() {
                return NewId::with_id(idx as u32 + 1);
            }
        }

        let id = self.objects.len() as u32;
        self.objects.push(None);
        NewId::with_id(id)
    }

    /// Attempts to bind a global. Panics if the global's version doesn't match
    /// our interface version.
    pub fn bind_global<I: Interface>(&mut self, name: u32, version: u32) -> Object<I>
    where
        NewId<I>: Into<SomeObject> + Into<Object<I>>,
    {
        println!("=> Binding `{}`", I::FFI_NAME);
        if version != I::VERSION {
            panic!("Wayland interface version mismatch:\nInterface: {}\nScaffolding's version: {}\nCompositor's version: {version}", I::FFI_NAME, I::VERSION)
        }

        let id = self.new_id();
        let registry = self.globals.registry;
        registry.bind(self, name, id.into());
        self.objects[id.id as usize] = Some(id.into());

        id.into()
    }

    pub fn create_callback(&mut self, func: WaylandCallback) -> NewId<Callback> {
        let id = self.new_id();
        self.objects[id.id as usize] = Some(SomeObject::Callback(id.into()));
        self.callbacks.push((id.id, func));

        id
    }
}
