use crate::Point;

/// Mouse buttons understood by the widget layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other,
}

/// Keyboard keys understood by the widget layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Enter,
    Space,
    Tab,
    Escape,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Character(char),
    Other,
}

/// Modifier state at the time of an event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Platform-independent input event.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    CursorMoved {
        position: Point,
    },
    MouseDown {
        position: Point,
        button: MouseButton,
    },
    MouseUp {
        position: Point,
        button: MouseButton,
    },
    CursorLeft,
    MouseWheel {
        position: Point,
        delta_x: f32,
        delta_y: f32,
    },
    KeyDown {
        key: Key,
        modifiers: Modifiers,
    },
    KeyUp {
        key: Key,
        modifiers: Modifiers,
    },
    TextInput(String),
    FocusGained,
    FocusLost,
}

impl Event {
    pub fn position(&self) -> Option<Point> {
        match self {
            Event::CursorMoved { position }
            | Event::MouseDown { position, .. }
            | Event::MouseUp { position, .. }
            | Event::MouseWheel { position, .. } => Some(*position),
            _ => None,
        }
    }

    pub fn is_pointer_event(&self) -> bool {
        matches!(
            self,
            Event::CursorMoved { .. }
                | Event::MouseDown { .. }
                | Event::MouseUp { .. }
                | Event::CursorLeft
                | Event::MouseWheel { .. }
        )
    }
}
