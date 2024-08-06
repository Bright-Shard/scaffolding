#![allow(dead_code)]

use {
    super::WaylandPlatform,
    std::{cell::Cell, convert::Infallible, fmt::Debug, io::Write, marker::PhantomData, mem},
};

use scaffolding::datatypes::ArenaVec;

// Wayland <-> Rust type map
// int: i32
// uint: u32
// fixed:
// string: String
// object: Object<I>
// new_id: NewId<I>, UntypedNewId
// array:
// fd:

pub struct Object<I: Interface> {
    pub id: u32,
    _ph: PhantomData<I>,
}
impl<I: Interface> Object<I> {
    pub fn with_id(id: u32) -> Self {
        Self {
            id,
            _ph: PhantomData,
        }
    }
}
impl<I: Interface> Debug for Object<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Object {{ id: {} }}", self.id)
    }
}
impl<I: Interface> Clone for Object<I> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<I: Interface> Copy for Object<I> {}

pub struct NewId<I: Interface> {
    pub id: u32,
    _ph: PhantomData<I>,
}
impl<I: Interface> NewId<I> {
    pub fn with_id(id: u32) -> Self {
        Self {
            id,
            _ph: PhantomData,
        }
    }
}
impl<I: Interface> From<NewId<I>> for Object<I> {
    fn from(value: NewId<I>) -> Self {
        Self {
            id: value.id,
            _ph: PhantomData,
        }
    }
}
impl<I: Interface> From<NewId<I>> for UntypedNewId<I> {
    fn from(value: NewId<I>) -> Self {
        Self {
            id: value.id,
            _ph: PhantomData,
        }
    }
}
impl<I: Interface> Debug for NewId<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NewId {{ id: {} }}", self.id)
    }
}
impl<I: Interface> Clone for NewId<I> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<I: Interface> Copy for NewId<I> {}

/// A [`NewId`] whose type is unknown by the compositor. When encoded to wire,
/// this type will encode its interface in addition to its object ID, unlike
/// [`NewId`] (which just encodes its object ID).
pub struct UntypedNewId<I: Interface> {
    pub id: u32,
    _ph: PhantomData<I>,
}
impl<I: Interface> UntypedNewId<I> {
    pub fn with_id(id: u32) -> Self {
        Self {
            id,
            _ph: PhantomData,
        }
    }
}
impl<I: Interface> From<UntypedNewId<I>> for Object<I> {
    fn from(value: UntypedNewId<I>) -> Self {
        Self {
            id: value.id,
            _ph: PhantomData,
        }
    }
}
impl<I: Interface> From<UntypedNewId<I>> for NewId<I> {
    fn from(value: UntypedNewId<I>) -> Self {
        Self {
            id: value.id,
            _ph: PhantomData,
        }
    }
}
impl<I: Interface> Debug for UntypedNewId<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NewId {{ id: {} }}", self.id)
    }
}
impl<I: Interface> Clone for UntypedNewId<I> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<I: Interface> Copy for UntypedNewId<I> {}

#[derive(Debug)]
#[repr(C)]
pub struct WireMsgHeader {
    /// The ID of an object. This message is either us calling a method of that
    /// object or the compositor sending an event of that object.
    pub object: u32,
    /// The ID of the method or event this message is calling.
    pub opcode: u16,
    /// The total size of this message, in bytes.
    pub len: u16,
}

pub struct WireDecoder<'a> {
    pub bytes: &'a [u8],
    progress: Cell<usize>,
}
impl<'a> WireDecoder<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            progress: Cell::new(8),
        }
    }

    pub fn header(&self) -> &'a WireMsgHeader {
        let ptr = self.bytes as *const [u8] as *const WireMsgHeader;
        unsafe { &*ptr }
    }

    pub fn decode<T: ReadWire<'a>>(&'a self) -> T {
        T::read_wire(self)
    }

    pub fn read_next(&self, bytes: usize) -> &'a [u8] {
        let progress = self.progress.get();
        let val = &self.bytes[progress..progress + bytes];
        self.progress.set(progress + bytes);

        val
    }
}

pub trait Interface {
    const VERSION: u32;
    const FFI_NAME: &'static str;

    type Error: Debug;
    type Opcode;
    type Event;
}

pub trait WriteWire {
    fn size(&self) -> u16;
    fn write_wire(self, msg_buffer: &ArenaVec<u8>);
}
mod write_wire_impls {
    use std::ffi::CStr;

    use scaffolding::utils;

    use super::*;

    impl<I: Interface> WriteWire for Object<I> {
        fn size(&self) -> u16 {
            4
        }
        fn write_wire(self, msg_buffer: &ArenaVec<u8>) {
            self.id.write_wire(msg_buffer);
        }
    }
    impl<I: Interface> WriteWire for NewId<I> {
        fn size(&self) -> u16 {
            4
        }
        fn write_wire(self, msg_buffer: &ArenaVec<u8>) {
            self.id.write_wire(msg_buffer);
        }
    }
    impl<I: Interface> WriteWire for UntypedNewId<I> {
        fn size(&self) -> u16 {
            I::FFI_NAME.size() + I::VERSION.size() + self.id.size()
        }
        fn write_wire(self, msg_buffer: &ArenaVec<u8>) {
            I::FFI_NAME.write_wire(msg_buffer);
            I::VERSION.write_wire(msg_buffer);
            self.id.write_wire(msg_buffer);
        }
    }
    impl WriteWire for u32 {
        fn size(&self) -> u16 {
            4
        }
        fn write_wire(self, msg_buffer: &ArenaVec<u8>) {
            for byte in self.to_ne_bytes() {
                msg_buffer.push(byte);
            }
        }
    }
    impl WriteWire for i32 {
        fn size(&self) -> u16 {
            4
        }
        fn write_wire(self, msg_buffer: &ArenaVec<u8>) {
            for byte in self.to_ne_bytes() {
                msg_buffer.push(byte);
            }
        }
    }
    impl WriteWire for &str {
        fn size(&self) -> u16 {
            // preceding u32, length of string, null byte
            4 + utils::align(self.len(), 4) as u16 + 1
        }
        fn write_wire(self, msg_buffer: &ArenaVec<u8>) {
            let len = self.len() + 1;
            (len as u32).write_wire(msg_buffer);
            for byte in self.as_bytes() {
                msg_buffer.push(*byte);
            }
            msg_buffer.push(b'\0');

            let align_diff = utils::align(len, 4) - len;
            for _ in 0..align_diff {
                msg_buffer.push(b'\0');
            }
        }
    }
    impl WriteWire for &CStr {
        fn size(&self) -> u16 {
            // preceding u32, length of string, null byte
            4 + self.count_bytes() as u16 + 1
        }
        fn write_wire(self, msg_buffer: &ArenaVec<u8>) {
            let len = self.count_bytes() + 1;
            (len as u32).write_wire(msg_buffer);
            for byte in self.to_bytes() {
                msg_buffer.push(*byte);
            }
            msg_buffer.push(b'\0');

            let align_diff = utils::align(len, 4) - len;
            for _ in 0..align_diff {
                msg_buffer.push(b'\0');
            }
        }
    }
}

pub trait ReadWire<'a> {
    fn read_wire(wire: &'a WireDecoder) -> Self;
}
mod read_wire_impls {
    use scaffolding::utils::align;

    use super::*;

    impl<'a> ReadWire<'a> for u32 {
        fn read_wire(wire: &'a WireDecoder) -> Self {
            let bytes: &[u8; 4] = wire.read_next(4).try_into().unwrap();
            u32::from_ne_bytes(*bytes)
        }
    }

    impl<'a> ReadWire<'a> for String {
        fn read_wire(wire: &'a WireDecoder) -> Self {
            let len = u32::read_wire(wire);
            // strings must be aligned to 4 bytes regardless of their actual
            // length in wire, so we have to read this many bytes, the rest
            // after length will just be null bytes
            let actual_len = align(len as usize, 4);
            let bytes = wire.read_next(actual_len);

            // trim off null bytes that are just there for alignment
            let trimmed = &bytes[..len as usize - 1];
            String::from_utf8_lossy(trimmed).to_string()
        }
    }
}

macro_rules! interfaces {
    ($(interface $struct_name:ident { version $version:literal; error $error:ty; name $ffi_name:ident; event $event_enum_name:ident; $(method $method_id:literal $method_name:ident($($arg_name:ident: $arg_ty:ty),*);)* $(event $event_id:literal $event_name:ident($($event_arg_name:ident: $event_arg_ty:ty),*);)* })*) => {
        $(
        mod $ffi_name {
            use super::*;

            #[allow(non_camel_case_types)]
            #[repr(u16)]
            pub enum Opcode {
                $($method_name = $method_id,)*
                // without this, declaring a interface with 0 methods will cause
                // an error
                // apparently you can't repr(u16) an empty enum, and if no
                // methods are added to the interface this enum will be empty,
                // so this just exists to avoid that error.
                _Invalid = u16::MAX
            }

            pub enum $event_enum_name {
                $($event_name {
                    $($event_arg_name: $event_arg_ty),*
                }),*
            }
            impl $event_enum_name {
                pub fn decode<'a>(decoder: &'a WireDecoder<'a>) -> Option<Self> {
                    match decoder.header().opcode {
                        $(
                        $event_id => Some(Self::$event_name {
                            $(
                                $event_arg_name: ReadWire::<'a>::read_wire(&decoder)
                            ),*
                        }),
                        )*
                        _ => {
                            eprintln!("Warning: Received unknown event from the Wayland compositor for `{}`", stringify!($event_enum_name));
                            None
                        }
                    }
                }
            }

            pub struct $struct_name;
            impl Interface for $struct_name {
                const VERSION: u32 = $version;
                const FFI_NAME: &'static str = stringify!($ffi_name);

                type Error = $error;
                type Opcode = Opcode;
                type Event = $event_enum_name;
            }

            impl Object<$struct_name> {
                $(
                    pub fn $method_name(&self, wl: &mut WaylandPlatform, $($arg_name: $arg_ty),*) {
                        let header = WireMsgHeader {
                            object: self.id,
                            opcode: $method_id,
                            len: $($arg_name.size() +)* 8
                        };

                        println!("<- Calling method {}::{}", stringify!($ffi_name), stringify!($method_name));
                        println!("  Header: {header:?}");
                        $(
                            println!("  Arg '{}': {:?}", stringify!($arg_name), $arg_name);
                        )*
                        let header_bytes: [u8; mem::size_of::<WireMsgHeader>()]
                            = unsafe { mem::transmute(header) };

                        wl.write_buffer.clear();
                        wl.write_buffer.extend(header_bytes);

                        // after the header is the opcode arguments
                        $(
                            $arg_name.write_wire(&wl.write_buffer);
                        )*

                        wl.compositor.write_all(&wl.write_buffer).unwrap();
                    }
                )*
            }

            impl From<Object<$struct_name>> for SomeObject {
                fn from(val: Object<$struct_name>) -> Self {
                    Self::$struct_name(val)
                }
            }
            impl From<NewId<$struct_name>> for SomeObject {
                fn from(val: NewId<$struct_name>) -> Self {
                    Self::$struct_name(val.into())
                }
            }
        }
        #[allow(unused_imports)]
        pub use $ffi_name::{$struct_name, $event_enum_name};
        )*

        pub enum SomeObject {
            $($struct_name(Object<$struct_name>)),*
        }
        pub enum SomeEvent {
            $($struct_name($ffi_name::$event_enum_name)),*
        }

        impl SomeObject {
            pub fn decode_event(&self, decoder: WireDecoder<'_>) -> Option<SomeEvent> {
                match self {
                    $(
                    Self::$struct_name(_) => {
                        Some(SomeEvent::$struct_name(<$struct_name as Interface>::Event::decode(&decoder)?))
                    }
                    )*
                }
            }
        }
    };
}

interfaces! {
    interface Display {
        version 1;
        error Infallible; // TODO
        name wl_display;
        event DisplayEvent;

        method 0 sync(callback: NewId<Callback>);
        method 1 get_registry(registry: NewId<Registry>);

        event 0 Error(object_id: u32, code: u32, message: String);
        event 1 DeleteId(id: u32);
    }
    interface Registry {
        version 1;
        error Infallible;
        name wl_registry;
        event RegistryEvent;

        method 0 bind(name: u32, id: UntypedNewId<impl Interface>);

        event 0 Global(name: u32, interface: String, version: u32);
        event 1 GlobalRemove(name: u32);
    }
    interface Callback {
        version 1;
        error Infallible;
        name wl_callback;
        event CallbackEvent;

        event 0 Done(callback_data: u32);
    }
    interface Surface {
        version 6;
        error Infallible; // TODO
        name wl_surface;
        event SurfaceEvent;
    }
    interface Shm {
        version 1;
        error Infallible; // TODO
        name wl_shm;
        event ShmEvent;

        event 0 Format(format: u32);
    }
    interface Compositor {
        version 6;
        error Infallible;
        name wl_compositor;
        event CompositorEvent;

        method 0 create_surface(id: NewId<Surface>);
    }

    interface XdgWmBase {
        version 6;
        error Infallible; // TODO
        name xdg_wm_base;
        event XdgWmBaseEvent;

        method 2 get_xdg_surface(id: NewId<XdgSurface>, surface: Object<Surface>);
        method 3 pong(serial: u32);

        event 0 Ping(serial: u32);
    }
    interface XdgSurface {
        version 6;
        error Infallible; // TODO
        name xdg_surface;
        event XdgSurfaceEvent;
    }
}
