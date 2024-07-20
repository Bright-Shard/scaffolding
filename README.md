# Scaffolding - a practical application framework

*Disclaimer: Scaffolding is **heavily** work-in-progress: this repo is more of a proof-of-concept than a useable project at the moment.*

Scaffolding is a modular and efficient app framework that's intended to solve common challenges from writing programs in Rust. Scaffolding isn't intended to be the most optimised or most technically impressive framework; it's just intended to be a suite of simple, practical tools to help you make your apps in less time. Currently, Scaffolding is under heavy development, but offers the following features:

1. **Automatic Data Management**: Scaffolding stores everything in a `World`. This puts all of the app's data in one location (or multiple, if you choose to use multiple `World`s). Special functions called *executables* - which are similar to systems in Bevy - can then query data from the `World` and mutate the `World` just by changing their arguments (similar to extractors in Axum).
2. **Simplified Mutability**: Let's be honest, the borrow checker is the archnemesis of just about any Rust project. Scaffolding takes a two-pronged approach to try and simplify mutability in Rust, to reduce some of the pains with the borrow checker:
  - **Custom Data Types**: Scaffolding introduces new data types that should simplify mutability in Rust, without sacrificing memory safety. For example, one of these types is the `ArenaVec` - a vector just like `Vec`, except its `push` method only needs `&self`, not `&mut self`. All data structures in Scaffolding also have documentation about how they work internally to maintain memory safety. You can find these structures in the `scaffolding::datatypes` module.
  - **Delayed Mutation**: Data that gets modified in the `World` doesn't get modified immediately. Instead, any changes to the `World` (known as *mutations*) get stored in a queue, which is then processed after a function is finished using the `World`. This prevents issues from mutable references, and is designed in a way that lets the compiler optimise out most of its overhead.
3. **Portable**: Scaffolding is a `no_std` library, and its only dependencies are platform-specific libraries to communicate with the OS (for example, it pulls in `libc` on Unix systems). Scaffolding defines a trait for OS APIs, and will run on any operating system where that trait is implemented. See [Adding Support for New Operating Systems](#adding-support-for-new-operating-systems) for more info.
4. **Modular**: Scaffolding is designed around plugin system, making the library itself quite small and its potential use cases quite large. External crates can easily use plugins to store data in the `World` and add their own APIs to Scaffolding.

By the way, Scaffolding is 100%

[![Human Made](https://humanmademark.com/white-logo.png)](https://humanmademark.com)

# What do you mean by "App Framework"? (or: future Scaffolding plans)

Scaffolding is meant to be a series of tools for developing apps, instead of just one UI library or build system. In particular, the following tools are planned for Scaffolding, to make app development even easier:

- **UI Library**: For GUI apps. UI elements will be functions just like everything else in Scaffolding, and thus will keep all of Scaffolding's advantages. App state can be shared between functions seamlessly, and the app's functionality can even be tested by testing `World` mutations.
- **Build System**: You'll be able to compile your app to any format, from a standalone executable (`AppImage`, `app`) to an installer (`dmg`, `msi`) to mobile formats (`ipa`, `apk`). Of course, this is Rust, so all of the normal executable formats will also be available (`exe`, `ELF`, `mach-o`, etc).
- **Debugger**: Scaffolding plans to have an integrated debugger, inspired by [the Tomorrow Corporation's tech demo](https://www.youtube.com/watch?v=72y2EC5fkcE). The debugger will be able to step through your app's functions, track mutations to the `World`, and allow you to undo or redo those mutations in real time.
- **Hot Reload**: Because everything's stored in the `World`, Scaffolding can quite easily dump and reload app state. This means it should be possible to dump the `World`, quickly recompile your app with Cranelift, and then reload the `World` in your new executable to essentially hot-reload your app.

# Platform Support

Scaffolding is currently only being developed on macOS and Linux. It plans to support macOS, Linux, Windows, iOS, and Android. Web may also be added at some point in the future.

## Adding Support for New Operating Systems

The `OsTrait` trait in `scaffolding::os` defines all of the OS functions Scaffolding relies on. It basically comes down to a few memory allocation functions. New operating systems just need to create an empty `Os` struct that implements `OsTrait` for Scaffolding to work correctly.

Scaffolding can be ported to any operating system with the following features:
- **Memory Allocation**: Scaffolding relies on a heap to store its app data.
- **Virtual Memory**: Scaffolding's `arenavec` type relies on virtual memory to guarantee that it will never move in memory. Most modern systems have memory paging, which provides virtual memory.
- **Atomics**: Scaffolding relies on atomic booleans for lazy loading and atomic pointers for multithreading.

# Project Status & TODO

Scaffolding isn't very useable right now because it doesn't have a UI library. Thus, the main focus of Scaffolding right now is its UI library.

<details>
<summary>Todo list</summary>

- [ ] World
  - [x] Plugins
    - [x] Can mutate the world and add arbitrary states
    - [x] Can load other plugins
    - [x] Only loaded once
  - [x] Mutations
    - [x] Can arbitrarily mutate the world
    - [x] Ability to invert/undo the mutation, allowing "step-forward"/"step-back" in a debugger
  - [x] ExecutableArg
    - [x] Get data from the world w/ references - have an arbitrary lifetime
    - [x] Apply mutations to the world when dropped
  - [x] Executables
    - [x] Take any argument that's `&impl ExecutableArg` or `&mut ExecutableArg`
    - [x] Apply mutations immediately or delayed
  - [ ] Multithreading
    - [x] `execute_in_parallel` function to run a bunch of executables in parallel, then apply all their mutations when they finish
    - [ ] Multithreading datatype: Double-buffered datatype for two threads to work with
    - [ ] Check `Send`/`Sync` impls for all types
  - [ ] Per-executable data
    - [ ] More planning...
    - Sometimes custom data needs to be passed to an executable, like function arguments.
    - This would currently require mutating data in the world, because executables can only access data from the world.
    - Currently using a model based on Swift's environments - a `TypeMap` is passed to each executable, and it can get data from that.
      - Downsides: Adds overhead and allocations, doesn't enforce that data is given to the executable (it could be accidentally left out of the typemap).
    - Alternative: `ExecutableWithArg<T>` trait, where T is one of the executable's arguments.
      - Example: `Fn(i32, &World)` would be an `ExecutableWithArg<i32>`.
      - Perks: Little (if any) overhead
      - Downsides: Due to compiler limitations, the argument would probably have to either be in the first position.
        - For example, this would be a `ExecutableWithArg<i32>`: `Fn(i32, &World)`, but this wouldn't: `Fn(&World, i32)`
        - This would cause confusion for users. Misplacing the argument would cause an extremely confusing `trait not satisfied` message.
  - [ ] Performance
    - Scaffolding's structure adds a lot of overhead. Executables have to get data from a `TypeMap` in the world instead of just getting that data from function arguments.
- [ ] Datatypes
  - [x] `TypeMap`: Like a `HashMap`, except the keys are types and the values are instances of those types
  - [x] `StackVec`: A vector whose first few elements are stored in an array on the stack
  - [x] `ArenaVec`: A vector guaranteed to never move in memory, allowing `push` to take `&self` instead of `&mut self`
  - [x] `Warehouse`: A vector that always moves values, and never borrows them, allowing for `remove` to take `&self` instead of `&mut self`
  - [ ] Think of more... ideally we'd never need to add more datatypes so we don't have to bump Scaffolding's version in the future
- [x] Standalone & Portable
  - [x] `no_std`
  - [x] Straightforward OS APIs, so it's easy to add support for new OSes
  - [x] Integration with std types
- [ ] UI
  - [ ] Windowing
    - This will start with just support for Wayland, since that's what my computer runs. It's also roughly in order of how I want to implement this.
    - [ ] Make a window appear
    - [ ] Draw a square (shape rendering below)
    - [ ] Mouse buttons
    - [ ] Keypresses
    - [ ] IME at some point
  - [ ] Shape Rendering
    - This will initially only use Vulkan. Eventually, Metal on macOS and DirectX on Windows should also be supported.
    - [ ] Triangles ~~by cYsmix~~
    - [ ] Squares
    - [ ] Rounded corners
    - [ ] Circles
  - [ ] Text Rendering
    - This may pull in dependencies for loading fonts initially. I haven't looked yet but that seems quite complicated.
  - [ ] Actual widgets
    - [ ] Button
    - [ ] Label
    - [ ] Text box (requires IME)
  - [ ] Widget settings
    - Need more planning... this should be things like colour and size
  - [ ] Layout
    - Also needs more planning
- [ ] Debugger
- [ ] Build System
- [ ] Hot Reload

</details>
