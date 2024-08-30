# Scaffolding - a practical application framework

Scaffolding is a modular and efficient app framework that's intended to solve common challenges from writing programs in Rust. Scaffolding isn't intended to be the most optimised or most technically impressive framework; it's just intended to be a suite of simple, practical tools to help you make your apps in less time. Currently, Scaffolding is under heavy development, but offers the following features:

1. **Automatic Data Management**: Scaffolding stores everything in a `World`. All of your app's data is stored in the `World`, and is easy to access with `Executable`s. `Executable`s are similar to systems in Bevy or request handlers in Axum - they're functions with special arguments that can query and modify any data in the `World`. This removes the need to constantly pass data to tons of functions in your app, which prevents borrow checker errors and removes the need for wrapper types like `Rc` or `RefCell`.
2. **Simplified Mutability**: Let's be honest, the borrow checker is the archnemesis of just about any Rust project. Scaffolding takes a two-pronged approach to try and simplify mutability in Rust, and therefore make the borrow checker easier to deal with:
  - **Custom Data Types**: Scaffolding introduces new data types that should simplify mutability in Rust, without sacrificing memory safety. For example, one of these types is the `ArenaVec` - a vector just like `Vec`, except its `push` method only needs `&self`, not `&mut self`. All data structures in Scaffolding also have documentation about how they work internally to maintain memory safety. You can find these structures in the `scaffolding::datatypes` module.
  - **Mutation with Message Passing**: `Executable`s can't change data in the `World` directly. Instead, `Executable`s send messages to the `World`, which will be processed after the `Executable` runs. This is based on the [Elm architecture](https://guide.elm-lang.org/architecture/). It also makes your app testable, as you can emulate many parts of it by sending messages to a test `World`.
3. **Portable**: Scaffolding is a `no_std` library, and its only dependencies are platform-specific libraries to communicate with the OS (for example, it pulls in `libc` on Unix systems). Scaffolding defines a trait for OS APIs, and will run on any operating system where that trait is implemented. See [Adding Support for New Operating Systems](#adding-support-for-new-operating-systems) for more info.
4. **Modular**: Scaffolding is designed around plugin system, making the library itself quite small and its potential use cases quite large. External crates can easily use plugins to store data in the `World` and add their own APIs to Scaffolding.

By the way, Scaffolding is 100%

[![Human Made](https://brainmade.org/white-logo.png)](https://brainmade.org)

# What do you mean by "App Framework"? (or: future Scaffolding plans)

Scaffolding is meant to be a series of tools for developing apps, instead of just one library. The core library has been intentionally designed to be extremely modular; everything else will build around it as a series of plugins. Thus, Scaffolding won't just be one crate; it'll be a framework of tools and libraries developed by me and, hopefully, other developers as well.

# Platform Support

Scaffolding is currently only being developed on macOS and Linux. By the time it's released, it will support macOS, Linux, Windows, iOS, and Android.

## Adding Support for New Operating Systems

The `OsTrait` trait in `scaffolding::os` defines all of the OS functions Scaffolding relies on. It basically comes down to a few memory allocation functions. New operating systems just need to create an empty `Os` struct that implements `OsTrait` for Scaffolding to work correctly.

Scaffolding can be ported to any operating system with the following features:
- **Memory Allocation**: Scaffolding relies on a heap to store its app data.
- **Virtual Memory**: Scaffolding's `arenavec` type relies on virtual memory to guarantee that it will never move in memory. Most modern systems have memory paging, which provides virtual memory.
- **Atomics**: Scaffolding relies on atomic booleans for lazy loading and atomic pointers for multithreading.

# Project Status & Roadmap

Scaffolding is under heavy development. Its API has been changed several times already, and will continue to change as I continue to experiment and improve it.

Currently, Scaffolding consists of the core library (`scaffolding`) and a TUI plugin (`scaffolding-tui`). I'm developing the TUI library because it's a good way to experiment with creating UIs in Scaffolding, while remaining a great deal simpler than a GUI library.

I plan on getting the TUI library to a state where it's comparable to other libraries, like Ratatui or Charm's libraries. The library is close to getting there; I need to cover some edge cases for mouse/keyboard input, add support for older terminals, and then add more widgets to the library.

After that, I plan on creating a GUI library with a similar API as my TUI library. The library will be powered by [Lokinit](https://github.com/loki-chat/lokinit), GPU-accelerated, and hopefully rely on less than 20 dependencies. The first versions of the GUI library will probably only support macOS and Linux.

Once both of those are done, I'll work on a debugger for Scaffolding. By the time I've finished the TUI and GUI libraries, Scaffolding should be in a state where it's demonstrably flexible enough for most use cases. The debugger will serve as a final test to make sure its plugin API is open enough to support something as invasive as a debugger.

By this point, the core Scaffolding library will be battle-tested enough that I'll release the first - and hopefully last - version of it. All that will be left after that is additional platform support for the GUI library, a build system for compiling into various platforms' native app formats (exe, appimage, app, ipa, apk, etc), and then more developer tools (such as hot reload).
