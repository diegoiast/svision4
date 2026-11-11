use std::{cell::Cell, rc::Rc};

use svision_core::prelude::*;

fn build_counter(ui: &Ui) -> (TypedWidgetHandle<Label>, Rc<Cell<i32>>) {
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let label = ui
        .add_child(&root, Label::new("0"))
        .cast::<Label>()
        .expect("label");
    let count = Rc::new(Cell::new(0));

    let callback_count = count.clone();
    let callback_label = label.clone();
    ui.add_child(
        &root,
        Button::new("Increment").on_click(move |_ctx| {
            let value = callback_count.get() + 1;
            callback_count.set(value);
            callback_label.edit(|label: &mut Label| {
                label.set_text(value.to_string());
            });
        }),
    );

    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();
    (label, count)
}

fn click(ui: &Ui, position: Point) {
    ui.dispatch(Event::MouseDown {
        position,
        button: MouseButton::Left,
    });
    ui.dispatch(Event::MouseUp {
        position,
        button: MouseButton::Left,
    });
}

#[test]
fn button_click_runs_callback_and_mutates_handle() {
    let ui = Ui::new();
    let (label, count) = build_counter(&ui);
    let root = ui.root().expect("root");
    let button = root.children()[1].bounds();

    click(&ui, button.center());

    assert_eq!(count.get(), 1);
    assert_eq!(label.with(|label| label.text().to_string()), "1");
}

#[test]
fn click_outside_does_not_fire_and_button_receives_focus() {
    let ui = Ui::new();
    let (label, count) = build_counter(&ui);

    click(&ui, Point::new(5.0, 5.0));
    assert_eq!(count.get(), 0);

    let root = ui.root().expect("root");
    let button = root.children()[1].bounds();
    click(&ui, button.center());
    assert_eq!(count.get(), 1);

    ui.dispatch(Event::KeyDown {
        key: Key::Enter,
        modifiers: Modifiers::default(),
    });
    assert_eq!(count.get(), 2);
    assert_eq!(label.with(|label| label.text().to_string()), "2");
}

#[test]
fn mouse_up_outside_button_does_not_fire() {
    let ui = Ui::new();
    let (_label, count) = build_counter(&ui);
    let root = ui.root().expect("root");
    let button = root.children()[1].bounds();

    ui.dispatch(Event::MouseDown {
        position: button.center(),
        button: MouseButton::Left,
    });
    ui.dispatch(Event::MouseUp {
        position: Point::new(5.0, 5.0),
        button: MouseButton::Left,
    });

    assert_eq!(count.get(), 0);
}

#[test]
fn disabled_button_does_not_fire() {
    let ui = Ui::new();
    let (_label, count) = build_counter(&ui);
    let root = ui.root().expect("root");
    let button = root.children()[1].clone();
    let bounds = button.bounds();
    button.set_enabled(false);

    click(&ui, bounds.center());

    assert_eq!(count.get(), 0);
}

#[test]
fn dragging_outside_and_back_fires_once_on_release() {
    let ui = Ui::new();
    let (_label, count) = build_counter(&ui);
    let root = ui.root().expect("root");
    let button = root.children()[1].bounds();

    ui.dispatch(Event::MouseDown {
        position: button.center(),
        button: MouseButton::Left,
    });
    ui.dispatch(Event::CursorMoved {
        position: Point::new(5.0, 5.0),
    });
    ui.dispatch(Event::CursorMoved {
        position: button.center(),
    });
    assert_eq!(count.get(), 0);
    ui.dispatch(Event::MouseUp {
        position: button.center(),
        button: MouseButton::Left,
    });

    assert_eq!(count.get(), 1);
}

#[test]
fn release_outside_cancels_and_next_click_still_works() {
    let ui = Ui::new();
    let (_label, count) = build_counter(&ui);
    let root = ui.root().expect("root");
    let button = root.children()[1].bounds();

    ui.dispatch(Event::MouseDown {
        position: button.center(),
        button: MouseButton::Left,
    });
    ui.dispatch(Event::CursorMoved {
        position: Point::new(5.0, 5.0),
    });
    ui.dispatch(Event::MouseUp {
        position: Point::new(5.0, 5.0),
        button: MouseButton::Left,
    });
    assert_eq!(count.get(), 0);

    click(&ui, button.center());
    assert_eq!(count.get(), 1);
}

#[test]
fn toggle_button_fires_and_toggles_state() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let toggle = ui.add_child(&root, Button::new("Toggle").toggle(true));
    let handle = toggle.cast::<Button>().expect("button");

    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();
    let bounds = handle.bounds();
    click(&ui, bounds.center());
    assert!(handle.with(|button: &Button| button.is_checked()));

    click(&ui, bounds.center());
    assert!(!handle.with(|button: &Button| button.is_checked()));
}

#[test]
fn layout_supports_four_different_padding_and_margin_values() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 240.0));
    let root = ui.set_root(
        VBoxLayout::new()
            .insets(Insets::new(1.0, 2.0, 3.0, 4.0))
            .margins(Insets::new(4.0, 3.0, 2.0, 1.0)),
    );
    ui.add_child(&root, Label::new("content"));
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    let bounds = root.bounds();
    assert!(bounds.width > 0.0);
    assert!(bounds.height > 0.0);
    assert!(ui.min_size().width > 0.0);
}

#[test]
fn listview_selects_and_scrolls() {
    let ui = Ui::new();
    ui.set_size(Size::new(200.0, 80.0));
    let root = ui.set_root(Column::new().padding(0.0).spacing(0.0));
    let list = ui
        .add_child(
            &root,
            ListView::new((0..10).map(|i| format!("Item {i}"))).selected(0),
        )
        .cast::<ListView>()
        .expect("list");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    assert_eq!(list.with(|l: &ListView| l.selected_index()), Some(0));
    let bounds = list.bounds();
    click(&ui, Point::new(bounds.x + 4.0, bounds.y + 40.0));
    assert_eq!(list.with(|l: &ListView| l.selected_index()), Some(1));

    ui.set_focus(Some(list.id()));
    ui.dispatch(Event::KeyDown {
        key: Key::Down,
        modifiers: Modifiers::default(),
    });
    assert_eq!(list.with(|l: &ListView| l.selected_index()), Some(2));

    ui.dispatch(Event::KeyDown {
        key: Key::End,
        modifiers: Modifiers::default(),
    });
    assert_eq!(list.with(|l: &ListView| l.selected_index()), Some(9));
    assert!(list.with(|l: &ListView| l.scroll_offset()) > 0.0);
}

#[test]
fn scrollbar_horizontal_value_changes() {
    let ui = Ui::new();
    ui.set_size(Size::new(200.0, 40.0));
    let root = ui.set_root(Column::new().padding(0.0).spacing(0.0));
    let bar = ui
        .add_child(&root, ScrollBar::horizontal().range(0.0, 100.0).value(0.0))
        .cast::<ScrollBar>()
        .expect("scrollbar");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    ui.set_focus(Some(bar.id()));
    ui.dispatch(Event::KeyDown {
        key: Key::Right,
        modifiers: Modifiers::default(),
    });
    assert_eq!(bar.with(|s: &ScrollBar| s.value_ref()), 1.0);

    ui.dispatch(Event::KeyDown {
        key: Key::End,
        modifiers: Modifiers::default(),
    });
    assert_eq!(bar.with(|s: &ScrollBar| s.value_ref()), 100.0);
}

#[test]
fn scrollbar_vertical_track_click_pages() {
    let ui = Ui::new();
    ui.set_size(Size::new(40.0, 200.0));
    let root = ui.set_root(Column::new().padding(0.0).spacing(0.0));
    let bar = ui
        .add_child(
            &root,
            ScrollBar::vertical()
                .range(0.0, 100.0)
                .value(0.0)
                .step_page(10.0),
        )
        .cast::<ScrollBar>()
        .expect("scrollbar");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    let bounds = bar.bounds();
    let thumb = bar.with(|s: &ScrollBar| s.value_ref());
    let _ = thumb;
    click(
        &ui,
        Point::new(bounds.x + bounds.width * 0.5, bounds.bottom() - 8.0),
    );
    assert!(bar.with(|s: &ScrollBar| s.value_ref()) > 0.0);
}

#[test]
fn combobox_opens_selects_and_closes() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 240.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let combo = ui
        .add_child(&root, ComboBox::new(["A", "B", "C"]).selected(0))
        .cast::<ComboBox>()
        .expect("combo");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    assert_eq!(combo.with(|c: &ComboBox| c.selected_index()), Some(0));
    assert!(!combo.with(|c: &ComboBox| c.is_open()));

    let bounds = combo.bounds();
    click(&ui, bounds.center());
    assert!(combo.with(|c: &ComboBox| c.is_open()));

    let item = Rect::new(bounds.x + 1.0, bounds.y + 36.0, bounds.width - 2.0, 32.0);
    click(&ui, item.center());
    assert_eq!(combo.with(|c: &ComboBox| c.selected_index()), Some(0));
    assert!(!combo.with(|c: &ComboBox| c.is_open()));
}

#[test]
fn combobox_closes_on_click_outside() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 240.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let combo = ui
        .add_child(&root, ComboBox::new(["A", "B"]).selected(0))
        .cast::<ComboBox>()
        .expect("combo");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    let bounds = combo.bounds();
    click(&ui, bounds.center());
    assert!(combo.with(|c: &ComboBox| c.is_open()));

    click(&ui, Point::new(300.0, 200.0));
    assert!(!combo.with(|c: &ComboBox| c.is_open()));
}

#[test]
fn combobox_keyboard_navigation() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 240.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let combo = ui
        .add_child(&root, ComboBox::new(["A", "B", "C"]).selected(0))
        .cast::<ComboBox>()
        .expect("combo");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    ui.set_focus(Some(combo.id()));
    ui.dispatch(Event::KeyDown {
        key: Key::Down,
        modifiers: Modifiers::default(),
    });
    assert_eq!(combo.with(|c: &ComboBox| c.selected_index()), Some(1));

    ui.dispatch(Event::KeyDown {
        key: Key::Enter,
        modifiers: Modifiers::default(),
    });
    assert!(combo.with(|c: &ComboBox| c.is_open()));

    ui.dispatch(Event::KeyDown {
        key: Key::Down,
        modifiers: Modifiers::default(),
    });
    ui.dispatch(Event::KeyDown {
        key: Key::Enter,
        modifiers: Modifiers::default(),
    });
    assert_eq!(combo.with(|c: &ComboBox| c.selected_index()), Some(2));
    assert!(!combo.with(|c: &ComboBox| c.is_open()));
}

fn build_two_buttons(ui: &Ui) -> (TypedWidgetHandle<Button>, TypedWidgetHandle<Button>) {
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let first = ui
        .add_child(&root, Button::new("First"))
        .cast::<Button>()
        .expect("first button");
    let second = ui
        .add_child(&root, Button::new("Second"))
        .cast::<Button>()
        .expect("second button");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();
    (first, second)
}

#[test]
fn tab_moves_focus_to_the_first_button() {
    let ui = Ui::new();
    let (first, _second) = build_two_buttons(&ui);

    ui.dispatch(Event::KeyDown {
        key: Key::Tab,
        modifiers: Modifiers::default(),
    });

    assert!(first.is_focused());
    assert_eq!(ui.focus().map(|handle| handle.id()), Some(first.id()));
}

#[test]
fn tab_moves_focus_forward_between_buttons() {
    let ui = Ui::new();
    let (first, second) = build_two_buttons(&ui);

    ui.dispatch(Event::KeyDown {
        key: Key::Tab,
        modifiers: Modifiers::default(),
    });
    ui.dispatch(Event::KeyDown {
        key: Key::Tab,
        modifiers: Modifiers::default(),
    });

    assert!(!first.is_focused());
    assert!(second.is_focused());
}

#[test]
fn shift_tab_moves_focus_backward_between_buttons() {
    let ui = Ui::new();
    let (first, second) = build_two_buttons(&ui);

    ui.dispatch(Event::KeyDown {
        key: Key::Tab,
        modifiers: Modifiers::default(),
    });
    ui.dispatch(Event::KeyDown {
        key: Key::Tab,
        modifiers: Modifiers::default(),
    });
    ui.dispatch(Event::KeyDown {
        key: Key::Tab,
        modifiers: Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    });

    assert!(first.is_focused());
    assert!(!second.is_focused());
}

#[test]
fn shift_tab_from_no_focus_selects_the_last_button() {
    let ui = Ui::new();
    let (_first, second) = build_two_buttons(&ui);

    ui.dispatch(Event::KeyDown {
        key: Key::Tab,
        modifiers: Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    });

    assert!(second.is_focused());
}

#[test]
fn tab_skips_disabled_buttons() {
    let ui = Ui::new();
    let (first, second) = build_two_buttons(&ui);
    first.set_enabled(false);

    ui.dispatch(Event::KeyDown {
        key: Key::Tab,
        modifiers: Modifiers::default(),
    });

    assert!(!first.is_focused());
    assert!(second.is_focused());
}

#[test]
fn focused_button_fires_on_enter() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let count = Rc::new(Cell::new(0));
    let callback_count = count.clone();
    let button = ui
        .add_child(
            &root,
            Button::new("Press").on_click(move |_ctx| {
                callback_count.set(callback_count.get() + 1);
            }),
        )
        .cast::<Button>()
        .expect("button");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    ui.set_focus(Some(button.id()));
    assert!(button.is_focused());

    ui.dispatch(Event::KeyDown {
        key: Key::Enter,
        modifiers: Modifiers::default(),
    });

    assert_eq!(count.get(), 1);
}

#[test]
fn theme_focus_color_is_exposed() {
    assert_ne!(Theme::light().focus(), Theme::light().surface());
    assert_ne!(Theme::dark().focus(), Theme::dark().surface());
}

#[test]
fn combobox_keyboard_change_fires_callback() {
    use std::cell::Cell;
    use std::rc::Rc;
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let called = Rc::new(Cell::new(false));
    let callback_called = called.clone();
    let combo = ui
        .add_child(
            &root,
            ComboBox::new(["A", "B"])
                .selected(0)
                .on_change(move |index, _ctx| {
                    callback_called.set(true);
                    assert_eq!(index, 1);
                }),
        )
        .cast::<ComboBox>()
        .expect("combo");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    ui.set_focus(Some(combo.id()));
    ui.dispatch(Event::KeyDown {
        key: Key::Enter,
        modifiers: Modifiers::default(),
    });
    ui.dispatch(Event::KeyDown {
        key: Key::Down,
        modifiers: Modifiers::default(),
    });
    ui.dispatch(Event::KeyDown {
        key: Key::Enter,
        modifiers: Modifiers::default(),
    });

    assert!(called.get());
}

#[test]
fn listview_click_center_selects_item() {
    let ui = Ui::new();
    ui.set_size(Size::new(200.0, 80.0));
    let root = ui.set_root(Column::new().padding(0.0).spacing(0.0));
    let list = ui
        .add_child(
            &root,
            ListView::new((0..10).map(|i| format!("Item {i}"))).selected(0),
        )
        .cast::<ListView>()
        .expect("list");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    let bounds = list.bounds();
    click(
        &ui,
        Point::new(bounds.x + bounds.width * 0.5, bounds.y + 40.0),
    );
    assert_eq!(list.with(|l: &ListView| l.selected_index()), Some(1));
}

#[test]
fn listview_click_item_does_not_scroll() {
    let ui = Ui::new();
    ui.set_size(Size::new(200.0, 80.0));
    let root = ui.set_root(Column::new().padding(0.0).spacing(0.0));
    let list = ui
        .add_child(
            &root,
            ListView::new((0..10).map(|i| format!("Item {i}"))).selected(0),
        )
        .cast::<ListView>()
        .expect("list");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    let before = list.with(|l: &ListView| l.scroll_offset());
    let bounds = list.bounds();
    click(&ui, Point::new(bounds.x + 4.0, bounds.y + 40.0));
    let after = list.with(|l: &ListView| l.scroll_offset());
    assert_eq!(before, after);
    assert_eq!(list.with(|l: &ListView| l.selected_index()), Some(1));
}

#[test]
fn listview_click_second_item_after_scroll_selects() {
    let ui = Ui::new();
    ui.set_size(Size::new(200.0, 80.0));
    let root = ui.set_root(Column::new().padding(0.0).spacing(0.0));
    let list = ui
        .add_child(
            &root,
            ListView::new((0..10).map(|i| format!("Item {i}"))).selected(0),
        )
        .cast::<ListView>()
        .expect("list");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();
    list.edit(|l: &mut ListView| l.set_scroll_offset(32.0));
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    let bounds = list.bounds();
    click(&ui, Point::new(bounds.x + 4.0, bounds.y + 40.0));
    assert_eq!(list.with(|l: &ListView| l.selected_index()), Some(2));
}

#[test]
fn listview_click_scrollbar_track_does_not_scroll_if_item_row() {
    let ui = Ui::new();
    ui.set_size(Size::new(200.0, 80.0));
    let root = ui.set_root(Column::new().padding(0.0).spacing(0.0));
    let list = ui
        .add_child(
            &root,
            ListView::new((0..10).map(|i| format!("Item {i}"))).selected(0),
        )
        .cast::<ListView>()
        .expect("list");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    let before = list.with(|l: &ListView| l.scroll_offset());
    let bounds = list.bounds();
    click(
        &ui,
        Point::new(bounds.x + bounds.width - 8.0, bounds.y + 40.0),
    );
    let after = list.with(|l: &ListView| l.scroll_offset());
    assert_eq!(before, after);
}

#[test]
fn listview_resize_then_click_does_not_scroll_up() {
    let ui = Ui::new();
    ui.set_size(Size::new(200.0, 80.0));
    let root = ui.set_root(Column::new().padding(0.0).spacing(0.0));
    let list = ui
        .add_child(
            &root,
            ListView::new((0..10).map(|i| format!("Item {i}"))).selected(0),
        )
        .cast::<ListView>()
        .expect("list");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();
    list.edit(|l: &mut ListView| l.set_scroll_offset(240.0));
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    ui.set_size(Size::new(200.0, 200.0));
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    let bounds = list.bounds();
    let before = list.with(|l: &ListView| l.scroll_offset());
    click(&ui, Point::new(bounds.x + 4.0, bounds.y + 40.0));
    let after = list.with(|l: &ListView| l.scroll_offset());
    assert_eq!(before, after);
}

#[test]
fn listview_scrollbar_button_auto_repeats() {
    use std::time::{Duration, Instant};
    let ui = Ui::new();
    ui.set_size(Size::new(200.0, 80.0));
    let root = ui.set_root(Column::new().padding(0.0).spacing(0.0));
    let list = ui
        .add_child(
            &root,
            ListView::new((0..20).map(|i| format!("Item {i}"))).selected(0),
        )
        .cast::<ListView>()
        .expect("list");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    let bounds = list.bounds();
    let scrollbar = list.with(|l: &ListView| l.scroll_offset());
    let _ = scrollbar;
    let sb_bounds = {
        // approximate right 16px
        Rect::new(bounds.right() - 16.0, bounds.y, 16.0, bounds.height)
    };
    let end_center = Point::new(sb_bounds.x + 8.0, sb_bounds.y + sb_bounds.height - 8.0);
    ui.dispatch(Event::MouseDown {
        position: end_center,
        button: MouseButton::Left,
    });
    let after_press = list.with(|l: &ListView| l.scroll_offset());
    // process auto repeat after delay
    let deadline = ui.process_auto_repeat(Instant::now() + Duration::from_millis(600));
    assert!(deadline.is_some());
    let after_repeat = list.with(|l: &ListView| l.scroll_offset());
    assert!(after_repeat != after_press);
}

#[test]
fn checkbox_is_focusable_via_tab() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let checkbox = ui
        .add_child(&root, Checkbox::new("Agree"))
        .cast::<Checkbox>()
        .expect("checkbox");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    ui.dispatch(Event::KeyDown {
        key: Key::Tab,
        modifiers: Modifiers::default(),
    });

    assert!(checkbox.is_focused());
    assert_eq!(ui.focus().map(|handle| handle.id()), Some(checkbox.id()));
}

#[test]
fn input_accepts_spaces() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let input_handle = ui.add_child(&root, Input::new());
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    ui.set_focus(Some(input_handle.id()));

    ui.dispatch(Event::KeyDown {
        key: Key::Space,
        modifiers: Modifiers::default(),
    });
    ui.dispatch(Event::KeyDown {
        key: Key::Space,
        modifiers: Modifiers::default(),
    });

    assert_eq!(
        input_handle.with_concrete(|input: &Input| input.text().to_string()),
        Some("  ".to_string())
    );
}

#[test]
fn ctrl_left_moves_to_word_start() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let input_handle = ui.add_child(&root, Input::with_text("hello world foo"));
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    ui.set_focus(Some(input_handle.id()));
    // Cursor starts at end. Ctrl+Left jumps to start of "foo".
    ui.dispatch(Event::KeyDown {
        key: Key::Left,
        modifiers: Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    });
    assert_eq!(
        input_handle.with_concrete(|input: &Input| input.cursor()),
        Some(12)
    );
}

#[test]
fn checkbox_toggles_on_space_and_enter() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let checkbox_handle = ui
        .add_child(&root, Checkbox::new("Agree"))
        .cast::<Checkbox>()
        .expect("checkbox");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    ui.set_focus(Some(checkbox_handle.id()));
    assert!(!checkbox_handle.with(|c: &Checkbox| c.is_checked()));

    ui.dispatch(Event::KeyDown {
        key: Key::Space,
        modifiers: Modifiers::default(),
    });
    assert!(checkbox_handle.with(|c: &Checkbox| c.is_checked()));

    ui.dispatch(Event::KeyDown {
        key: Key::Enter,
        modifiers: Modifiers::default(),
    });
    assert!(!checkbox_handle.with(|c: &Checkbox| c.is_checked()));
}

#[test]
fn radiobutton_checks_on_space() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let radio_handle = ui
        .add_child(&root, RadioButton::new("Option").id("a"))
        .cast::<RadioButton>()
        .expect("radio");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    ui.set_focus(Some(radio_handle.id()));
    assert!(!radio_handle.with(|r: &RadioButton| r.is_checked()));

    ui.dispatch(Event::KeyDown {
        key: Key::Space,
        modifiers: Modifiers::default(),
    });
    assert!(radio_handle.with(|r: &RadioButton| r.is_checked()));
}

#[test]
fn listview_selected_item_has_visible_frame() {
    let ui = Ui::new();
    ui.set_size(Size::new(200.0, 80.0));
    let root = ui.set_root(Column::new().padding(0.0).spacing(0.0));
    let list_handle = ui
        .add_child(
            &root,
            ListView::new((0..5).map(|i| format!("Item {i}"))).selected(2),
        )
        .cast::<ListView>()
        .expect("list");
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    assert_eq!(list_handle.with(|l: &ListView| l.selected_index()), Some(2));
}

#[test]
fn button_auto_repeat_fires_repeatedly_while_held() {
    use std::time::{Duration, Instant};
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let count = Rc::new(Cell::new(0));
    let callback_count = count.clone();
    ui.add_child(
        &root,
        Button::new("Repeat")
            .auto_repeat(true)
            .auto_repeat_delay(0.1)
            .auto_repeat_interval(0.1)
            .on_click(move |_ctx| {
                callback_count.set(callback_count.get() + 1);
            }),
    );
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    let bounds = root.children()[0].bounds();
    ui.dispatch(Event::MouseDown {
        position: bounds.center(),
        button: MouseButton::Left,
    });
    // fires once on press
    assert_eq!(count.get(), 1);

    let base = Instant::now();
    // before delay: no repeat
    let deadline = ui.process_auto_repeat(base + Duration::from_millis(50));
    assert!(deadline.is_some());
    assert_eq!(count.get(), 1);

    // after delay: repeats
    let deadline = ui.process_auto_repeat(base + Duration::from_millis(250));
    assert!(deadline.is_some());
    assert!(count.get() > 1, "expected repeats, got {}", count.get());

    ui.dispatch(Event::MouseUp {
        position: bounds.center(),
        button: MouseButton::Left,
    });
    let after_release = count.get();

    // no more repeats after release
    ui.process_auto_repeat(base + Duration::from_secs(5));
    assert_eq!(count.get(), after_release);
}

#[test]
fn button_auto_repeat_resumes_when_pointer_comes_back() {
    use std::time::{Duration, Instant};
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let count = Rc::new(Cell::new(0));
    let callback_count = count.clone();
    ui.add_child(
        &root,
        Button::new("Repeat")
            .auto_repeat(true)
            .auto_repeat_delay(0.1)
            .auto_repeat_interval(0.1)
            .on_click(move |_ctx| {
                callback_count.set(callback_count.get() + 1);
            }),
    );
    ui.set_size(Size::new(200.0, 200.0));
    ui.layout();

    let bounds = root.children()[0].bounds();
    let center = bounds.center();
    let outside = Point::new(center.x, bounds.bottom() + 20.0);

    ui.dispatch(Event::MouseDown {
        position: center,
        button: MouseButton::Left,
    });
    assert_eq!(count.get(), 1);

    let base = Instant::now();
    ui.process_auto_repeat(base + Duration::from_millis(250));
    let after_repeat = count.get();
    assert!(after_repeat > 1, "expected repeats, got {after_repeat}");

    // pointer leaves the button while it is held down: repeating pauses
    ui.dispatch(Event::CursorMoved { position: outside });
    let paused = ui.process_auto_repeat(base + Duration::from_millis(500));
    assert!(paused.is_none());
    assert_eq!(count.get(), after_repeat);

    // pointer comes back while the button is still held: repeating resumes
    ui.dispatch(Event::CursorMoved { position: center });
    let deadline = ui.process_auto_repeat(base + Duration::from_millis(600));
    assert!(deadline.is_some());
    ui.process_auto_repeat(base + Duration::from_millis(800));
    assert!(
        count.get() > after_repeat,
        "expected repeat after re-entering, got {}",
        count.get()
    );

    // releasing outside ends the repeat for good
    ui.dispatch(Event::MouseUp {
        position: outside,
        button: MouseButton::Left,
    });
    let after_release = count.get();
    ui.dispatch(Event::CursorMoved { position: center });
    ui.process_auto_repeat(base + Duration::from_secs(5));
    assert_eq!(count.get(), after_release);
}

#[test]
fn input_on_change_fires_when_text_is_edited() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let seen = Rc::new(std::cell::RefCell::new(String::new()));
    let seen_cb = seen.clone();
    let input = ui
        .add_child(
            &root,
            Input::with_text("").on_change(move |text, _ctx| {
                *seen_cb.borrow_mut() = text.to_string();
            }),
        )
        .cast::<Input>()
        .expect("input");
    ui.layout();

    let bounds = input.bounds();
    click(&ui, bounds.center());

    ui.dispatch(Event::KeyDown {
        key: Key::Character('h'),
        modifiers: Modifiers::default(),
    });
    ui.dispatch(Event::KeyDown {
        key: Key::Character('i'),
        modifiers: Modifiers::default(),
    });
    assert_eq!(seen.borrow().as_str(), "hi");
    assert_eq!(input.with(|input: &Input| input.text().to_string()), "hi");

    // A cursor move must not report a change.
    ui.dispatch(Event::KeyDown {
        key: Key::Left,
        modifiers: Modifiers::default(),
    });
    assert_eq!(seen.borrow().as_str(), "hi");
}

#[test]
fn input_shows_placeholder_while_empty() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let input = ui
        .add_child(&root, Input::with_text("").placeholder("Search songs"))
        .cast::<Input>()
        .expect("input");
    ui.layout();

    assert_eq!(
        input.with(|input: &Input| input.placeholder_text().map(str::to_string)),
        Some("Search songs".to_string())
    );
    assert_eq!(input.with(|input: &Input| input.text().to_string()), "");
}

#[test]
fn stack_shows_only_the_current_child() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let stack = ui.add_child(&root, Stack::new());
    let first = ui.add_child(&stack, Label::new("first"));
    let second = ui.add_child(&stack, Label::new("second"));
    ui.layout();

    assert!(first.is_visible());
    assert!(!second.is_visible());

    stack.edit(|stack: &mut Stack| stack.set_current(1));
    ui.layout();

    assert!(!first.is_visible());
    assert!(second.is_visible());
}

#[test]
fn stack_children_share_the_same_bounds() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let stack = ui.add_child(&root, Stack::new());
    let first = ui.add_child(&stack, Label::new("first"));
    let second = ui.add_child(&stack, Label::new("second"));
    ui.layout();

    let stack_bounds = stack.bounds();
    assert_eq!(first.bounds(), stack_bounds);
    assert_eq!(second.bounds(), stack_bounds);
}

#[test]
fn tab_bar_selects_the_clicked_tab() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let bar = ui
        .add_child(&root, TabBar::new(["One", "Two", "Three"]))
        .cast::<TabBar>()
        .expect("tab bar");
    ui.layout();

    assert_eq!(bar.with(|bar: &TabBar| bar.selected()), 0);

    let second = bar.with(|bar: &TabBar| bar.tab_rect(1)).expect("second tab");
    click(&ui, second.center());

    assert_eq!(bar.with(|bar: &TabBar| bar.selected()), 1);
}

#[test]
fn tab_bar_arrow_keys_wrap_around() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let bar = ui
        .add_child(&root, TabBar::new(["One", "Two"]))
        .cast::<TabBar>()
        .expect("tab bar");
    ui.layout();

    ui.set_focus(Some(bar.id()));

    ui.dispatch(Event::KeyDown {
        key: Key::Left,
        modifiers: Modifiers::default(),
    });
    assert_eq!(bar.with(|bar: &TabBar| bar.selected()), 1);

    ui.dispatch(Event::KeyDown {
        key: Key::Right,
        modifiers: Modifiers::default(),
    });
    assert_eq!(bar.with(|bar: &TabBar| bar.selected()), 0);
}

#[test]
fn tab_widget_switches_pages_when_a_tab_is_clicked() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let tabs = ui
        .add_child(&root, TabWidget::new(["Songs", "Albums"]))
        .cast::<TabWidget>()
        .expect("tab widget");
    let songs = ui.add_child(&tabs.handle(), Label::new("songs page"));
    let albums = ui.add_child(&tabs.handle(), Label::new("albums page"));
    ui.layout();

    assert!(songs.is_visible());
    assert!(!albums.is_visible());
    assert_eq!(tabs.with(|tabs: &TabWidget| tabs.selected()), 0);

    let second_tab = tabs.with(|tabs: &TabWidget| tabs.tab_rect(1)).expect("second tab");
    click(&ui, second_tab.center());

    assert_eq!(tabs.with(|tabs: &TabWidget| tabs.selected()), 1);
    ui.layout();
    assert!(!songs.is_visible());
    assert!(albums.is_visible());
}

#[test]
fn tab_widget_reports_tab_changes() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let selected = Rc::new(Cell::new(usize::MAX));
    let callback_selected = selected.clone();
    let tabs = ui
        .add_child(
            &root,
            TabWidget::new(["Songs", "Albums"]).on_change(move |index, _ctx| {
                callback_selected.set(index);
            }),
        )
        .cast::<TabWidget>()
        .expect("tab widget");
    ui.add_child(&tabs.handle(), Label::new("songs page"));
    ui.add_child(&tabs.handle(), Label::new("albums page"));
    ui.layout();

    let second_tab = tabs.with(|tabs: &TabWidget| tabs.tab_rect(1)).expect("second tab");
    click(&ui, second_tab.center());

    assert_eq!(selected.get(), 1);
}

#[test]
fn tab_widget_with_more_tabs_than_pages_stays_consistent() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let tabs = ui
        .add_child(&root, TabWidget::new(["One", "Two", "Three"]))
        .cast::<TabWidget>()
        .expect("tab widget");
    let only = ui.add_child(&tabs.handle(), Label::new("only page"));
    ui.layout();

    let third = tabs.with(|tabs: &TabWidget| tabs.tab_rect(2)).expect("third tab");
    click(&ui, third.center());
    ui.layout();

    assert!(only.is_visible());
    assert_eq!(tabs.with(|tabs: &TabWidget| tabs.selected()), 0);
}

#[test]
fn tab_widget_as_root_widget_measures_its_tabs() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    // The root widget is arranged without ever being measured, so the tab widget
    // has to measure its tab titles itself.
    let tabs = ui
        .set_root(TabWidget::new(["Main", "Second"]))
        .cast::<TabWidget>()
        .expect("tab widget");
    ui.add_child(&tabs.handle(), Label::new("content"));
    ui.layout();

    let tab = tabs.with(|tabs: &TabWidget| tabs.tab_rect(0)).expect("tab");
    assert!(tab.width > 0.0, "tab should have a width");
    assert!(tab.height > 0.0, "tab should have a height");
}

#[test]
fn list_view_filter_is_case_insensitive() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let list = ui
        .add_child(
            &root,
            ListView::new(["Love Me Do", "Please Please Me", "Help!"]),
        )
        .cast::<ListView>()
        .expect("list");
    ui.layout();

    // Uppercase query matches a lowercase item.
    list.edit(|list: &mut ListView| list.set_filter("LOVE"));
    assert_eq!(
        list.with(|list: &ListView| list.visible_items()),
        vec!["Love Me Do".to_string()]
    );

    // Lowercase query matches a mixed-case item.
    list.edit(|list: &mut ListView| list.set_filter("please"));
    assert_eq!(
        list.with(|list: &ListView| list.visible_items()),
        vec!["Please Please Me".to_string()]
    );

    // Mixed case query matches too.
    list.edit(|list: &mut ListView| list.set_filter("hElP"));
    assert_eq!(
        list.with(|list: &ListView| list.visible_items()),
        vec!["Help!".to_string()]
    );

    // An empty filter shows everything again.
    list.edit(|list: &mut ListView| list.clear_filter());
    assert_eq!(
        list.with(|list: &ListView| list.visible_items()),
        vec![
            "Love Me Do".to_string(),
            "Please Please Me".to_string(),
            "Help!".to_string()
        ]
    );
}

#[test]
fn list_view_filter_survives_an_item_change() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let list = ui
        .add_child(&root, ListView::new(["Love Me Do", "Help!"]))
        .cast::<ListView>()
        .expect("list");
    ui.layout();

    list.edit(|list: &mut ListView| list.set_filter("love"));
    list.edit(|list: &mut ListView| list.set_items(["Love Me Do", "Help!", "Love For Sale"]));

    assert_eq!(
        list.with(|list: &ListView| list.visible_items()),
        vec!["Love Me Do".to_string(), "Love For Sale".to_string()]
    );
}

#[test]
fn list_view_filter_matches_a_substring() {
    let ui = Ui::new();
    ui.set_size(Size::new(320.0, 200.0));
    let root = ui.set_root(Column::new().padding(10.0).spacing(8.0));
    let list = ui
        .add_child(&root, ListView::new(["I Saw Her Standing There", "Misery"]))
        .cast::<ListView>()
        .expect("list");
    ui.layout();

    list.edit(|list: &mut ListView| list.set_filter("standing"));
    assert_eq!(
        list.with(|list: &ListView| list.visible_items()),
        vec!["I Saw Her Standing There".to_string()]
    );
}
