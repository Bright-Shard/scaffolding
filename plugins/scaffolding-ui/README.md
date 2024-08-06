# ScaffoldingUI - A practical UI library for Scaffolding

ScaffoldingUI is a UI library for the Scaffolding application framework.

Note: ScaffoldingUI isn't as portable as Scaffolding. It does require the standard library, and has an additional set of functions that have to be implemented for it to work on an operating system.

# Code Layout

The core of ScaffoldingUI is the `Display`. The `Display` is the bridge between Scaffolding and platform-specific APIs for creating GUI applications.
