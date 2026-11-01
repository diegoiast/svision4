mod button;
mod column;
mod combobox;
mod label;
mod listview;
pub mod checkbox;
pub mod radiobutton;
mod row;
mod scrollbar;
mod spacer;

pub use button::Button;
pub type HBoxLayout = Row;
pub type VBoxLayout = Column;

pub use checkbox::Checkbox;
pub use column::Column;
pub use combobox::ComboBox;
pub use label::Label;
pub use listview::ListView;
pub use radiobutton::{RadioButton, RadioButtonGroup};
pub use row::Row;
pub use scrollbar::{Orientation, ScrollBar};
pub use spacer::Spacer;

/// Alignment along either axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

/// Alignment of a container's children on its cross axis.
pub type CrossAlign = Align;

/// Horizontal text alignment inside a widget's bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}
