use crate::Key;

/// Interaction state shared by buttons, checkboxes and radio buttons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonState {
    #[default]
    Normal,
    Hovered,
    ClickedInside,
    ClickedOutside,
}

/// Port of `toolkit::ButtonStateHandler`.
///
/// The handler only tracks interaction. Visual invalidation is signalled by the
/// `bool` returned from [`on_mouse_enter`](Self::on_mouse_enter) and
/// [`on_mouse_leave`](Self::on_mouse_leave); [`on_mouse_click`](Self::on_mouse_click)
/// returns whether the event was handled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ButtonStateHandler {
    pub state: ButtonState,
}

impl ButtonStateHandler {
    pub const fn new() -> Self {
        Self {
            state: ButtonState::Normal,
        }
    }

    pub fn reset(&mut self) -> bool {
        if self.state == ButtonState::Normal {
            false
        } else {
            self.state = ButtonState::Normal;
            true
        }
    }

    pub fn on_mouse_enter(&mut self) -> bool {
        match self.state {
            ButtonState::ClickedOutside => {
                self.state = ButtonState::ClickedInside;
                true
            }
            ButtonState::Hovered => false,
            ButtonState::Normal => {
                self.state = ButtonState::Hovered;
                true
            }
            ButtonState::ClickedInside => false,
        }
    }

    pub fn on_mouse_leave(&mut self) -> bool {
        match self.state {
            ButtonState::ClickedInside => {
                self.state = ButtonState::ClickedOutside;
                true
            }
            ButtonState::ClickedOutside => false,
            ButtonState::Hovered => {
                self.state = ButtonState::Normal;
                true
            }
            ButtonState::Normal => false,
        }
    }

    pub fn on_mouse_click(&mut self, pressed: bool) -> bool {
        match self.state {
            ButtonState::ClickedInside => {
                if !pressed {
                    self.state = ButtonState::Hovered;
                }
                true
            }
            ButtonState::ClickedOutside => {
                if pressed {
                    self.state = ButtonState::ClickedInside;
                    true
                } else {
                    self.state = ButtonState::Normal;
                    false
                }
            }
            ButtonState::Hovered => {
                if pressed {
                    self.state = ButtonState::ClickedInside;
                }
                pressed
            }
            ButtonState::Normal => {
                if pressed {
                    self.state = ButtonState::ClickedInside;
                }
                pressed
            }
        }
    }

    pub fn on_keyboard(&self, key: Key, pressed: bool) -> bool {
        pressed && matches!(key, Key::Enter | Key::Tab)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_and_leave_follow_the_state_machine() {
        let mut handler = ButtonStateHandler::new();
        assert_eq!(handler.state, ButtonState::Normal);

        assert!(handler.on_mouse_enter());
        assert_eq!(handler.state, ButtonState::Hovered);
        assert!(!handler.on_mouse_enter());
        assert_eq!(handler.state, ButtonState::Hovered);

        assert!(handler.on_mouse_leave());
        assert_eq!(handler.state, ButtonState::Normal);
        assert!(!handler.on_mouse_leave());
    }

    #[test]
    fn press_transitions_to_clicked_inside() {
        let mut handler = ButtonStateHandler::new();
        handler.on_mouse_enter();

        assert!(handler.on_mouse_click(true));
        assert_eq!(handler.state, ButtonState::ClickedInside);

        assert!(handler.on_mouse_click(false));
        assert_eq!(handler.state, ButtonState::Hovered);
    }

    #[test]
    fn drag_outside_and_back_tracks_pressed_state() {
        let mut handler = ButtonStateHandler::new();
        handler.on_mouse_enter();
        handler.on_mouse_click(true);

        assert!(handler.on_mouse_leave());
        assert_eq!(handler.state, ButtonState::ClickedOutside);

        assert!(handler.on_mouse_enter());
        assert_eq!(handler.state, ButtonState::ClickedInside);

        assert!(handler.on_mouse_click(false));
        assert_eq!(handler.state, ButtonState::Hovered);
    }

    #[test]
    fn release_outside_cancels_the_click() {
        let mut handler = ButtonStateHandler::new();
        handler.on_mouse_enter();
        handler.on_mouse_click(true);
        handler.on_mouse_leave();

        assert!(!handler.on_mouse_click(false));
        assert_eq!(handler.state, ButtonState::Normal);
    }

    #[test]
    fn release_without_press_is_ignored() {
        let mut handler = ButtonStateHandler::new();
        assert!(!handler.on_mouse_click(false));
        assert_eq!(handler.state, ButtonState::Normal);

        handler.on_mouse_enter();
        assert!(!handler.on_mouse_click(false));
        assert_eq!(handler.state, ButtonState::Hovered);
    }

    #[test]
    fn enter_from_clicked_outside_restores_clicked_inside() {
        let mut handler = ButtonStateHandler::new();
        handler.on_mouse_click(true);
        assert_eq!(handler.state, ButtonState::ClickedInside);

        handler.on_mouse_leave();
        assert_eq!(handler.state, ButtonState::ClickedOutside);

        assert!(handler.on_mouse_enter());
        assert_eq!(handler.state, ButtonState::ClickedInside);
    }

    #[test]
    fn keyboard_follows_enter_and_tab() {
        let handler = ButtonStateHandler::new();
        assert!(handler.on_keyboard(Key::Enter, true));
        assert!(handler.on_keyboard(Key::Tab, true));
        assert!(!handler.on_keyboard(Key::Space, true));
        assert!(!handler.on_keyboard(Key::Enter, false));
    }
}
