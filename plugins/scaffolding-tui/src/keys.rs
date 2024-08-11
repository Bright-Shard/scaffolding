#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum Key {
    // a, b, c, etc. or か
    Text(char),

    // arrows
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    // misc
    Escape,
    Delete,
}
