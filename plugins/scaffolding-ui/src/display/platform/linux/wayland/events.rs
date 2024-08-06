use {
    super::{wire::*, WaylandPlatform},
    std::io::{ErrorKind, Read},
};

impl WaylandPlatform {
    /// Reads from the compositor and processes the next received event from it.
    pub fn process_next_event(&mut self, block_thread: bool) {
        self.read_buffer.resize(8, 0);

        self.compositor.set_nonblocking(!block_thread).unwrap();
        if let Err(err) = self.compositor.read_exact(&mut self.read_buffer) {
            if !block_thread && err.kind() == ErrorKind::WouldBlock {
                return;
            }

            panic!("Failed to read from the Wayland compositor: {err:?}");
        }

        let header = &self.read_buffer as &[u8] as *const [u8] as *const WireMsgHeader;
        let header = unsafe { &*header };

        self.read_buffer.resize(header.len as usize, 0);
        self.compositor
            .read_exact(&mut self.read_buffer[8..])
            .expect("Failed to read message from Wayland compositor");

        let decoder = WireDecoder::new(&self.read_buffer);
        let object_id = decoder.header().object;
        let object = self.objects[object_id as usize]
            .as_ref()
            .expect("Wayland compositor sent a message for an invalid object");

        println!("\n\n-> Got event: {:?}", decoder.header());

        let Some(event) = object.decode_event(decoder) else {
            return;
        };

        match event {
            SomeEvent::Callback(event) => match event {
                CallbackEvent::Done { callback_data } => {
                    let cb = self
                        .callbacks
                        .iter()
                        .enumerate()
                        .find(|(_, (id, _))| *id == object_id);

                    if let Some((callbacks_idx, _)) = cb {
                        let (_, func) = self.callbacks.remove(callbacks_idx).unwrap();
                        func(self, callback_data);
                    } else {
                        println!("WARNING: cb was None");
                    }
                }
            },
            SomeEvent::Display(event) => match event {
                DisplayEvent::Error {
                    object_id,
                    code,
                    message,
                } => {
                    panic!("Received a critical error from the Wayland compositor!\nMessage: '{message}'\nError code: {code}\nResponsible object: {object_id}");
                }
                DisplayEvent::DeleteId { id } => {
                    println!("Removing object with id {id}");
                    self.objects[id as usize] = None;
                }
            },
            SomeEvent::Registry(event) => match event {
                RegistryEvent::Global {
                    name,
                    interface,
                    version,
                } => {
                    println!("Discovered global {interface}@v{version} (name: {name})");
                    match interface.as_str() {
                        "xdg_wm_base" => {
                            self.globals.xdg_wm_base = self.bind_global(name, version);
                        }
                        "wl_shm" => {
                            self.globals.shm = self.bind_global(name, version);
                        }
                        "wl_compositor" => {
                            self.globals.compositor = self.bind_global(name, version);
                        }
                        _ => {}
                    }
                }
                RegistryEvent::GlobalRemove { name } => {
                    println!("Global {name} was removed");
                }
            },
            SomeEvent::Shm(event) => match event {
                ShmEvent::Format { format: _ } => {
                    // We only use ARGB, which is required by Wayland, so we
                    // don't need this event
                }
            },
            _ => {
                eprintln!("WARNING: Not processing event for object {object_id}");
            }
        }
    }
}
