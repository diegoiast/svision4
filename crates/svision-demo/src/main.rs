use std::{cell::Cell, error::Error, rc::Rc};

use svision_backend_softbuffer::SoftbufferBackend;
use svision_core::prelude::*;
use svision_raster_soft::SoftRasterizer;
use svision_winit::{WindowConfig, run};

mod beatles;

fn main() -> Result<(), Box<dyn Error>> {
    if let Some(path) = screenshot_path() {
        return run_screenshot(&path);
    }

    let config = WindowConfig::default()
        .title("svision - retained counter")
        .size(900.0, 560.0);

    #[cfg(windows)]
    if std::env::args().any(|arg| arg == "--gdiplus") {
        return run::<svision_backend_gdiplus::GdiPlusBackend, _>(config, build);
    }
    //return run::<svision_backend_gdiplus::GdiPlusBackend, _>(config, build);
    run::<SoftbufferBackend, _>(config, build)
}

fn screenshot_path() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--screenshot" || arg == "-s" {
            return args.next();
        }
        if !arg.starts_with("--") && arg != "--gdiplus" {
            return Some(arg);
        }
    }
    None
}

fn run_screenshot(path: &str) -> Result<(), Box<dyn Error>> {
    let ui = Ui::new();
    ui.set_size(Size::new(900.0, 560.0));
    build(&ui);
    ui.layout();

    let root = ui.root().expect("root");
    // The buttons live in a card on the tab widget's first page. Find the row by
    // looking for the "Increment" button instead of hard-coding an index.
    let page = root.children()[0].clone();
    let increment = find_button(&page, "Increment").expect("increment button");
    let buttons = increment.parent().expect("button row");
    let button_count = buttons.child_count();

    let mut rasterizer = SoftRasterizer::new();
    ui.paint(&mut rasterizer);
    let pixmap = rasterizer.pixmap();
    pixmap.save_png(path)?;

    let background = ui.theme().background();
    let mut visible_buttons = 0;
    for button in buttons.children() {
        let bounds = button.bounds();
        let x = (bounds.x + bounds.width * 0.5).round() as usize;
        let y = (bounds.y + bounds.height * 0.5).round() as usize;
        if x < pixmap.width() as usize && y < pixmap.height() as usize {
            let index = (y * pixmap.width() as usize + x) * 4;
            let data = pixmap.data();
            if data[index] != background.r
                || data[index + 1] != background.g
                || data[index + 2] != background.b
            {
                visible_buttons += 1;
            }
        }
    }

    println!(
        "saved screenshot to {path} ({}x{}), buttons={button_count}, visible_buttons={visible_buttons}",
        pixmap.width(),
        pixmap.height()
    );
    Ok(())
}

/// Depth-first search for a button with the given label.
fn find_button(parent: &WidgetHandle, label: &str) -> Option<WidgetHandle> {
    for child in parent.children() {
        let is_match = child
            .with_concrete::<Button, _>(|button| button.text() == label)
            .unwrap_or(false);
        if is_match {
            return Some(child);
        }
        if let Some(found) = find_button(&child, label) {
            return Some(found);
        }
    }
    None
}

fn build(ui: &Ui) {
    // The window is a tab widget with two tabs: the counter on the first page
    // and the Beatles browser on the second.
    let tabs = ui
        .set_root(TabWidget::new(["Main", "Beatles"]))
        .cast::<TabWidget>()
        .expect("root tab widget");
    let root = ui.add_child(&tabs.handle(), VBoxLayout::new());

    ui.add_child(&root, Label::new("svision4").font_size(30.0).bold());
    ui.add_child(&root, Label::new("SVision4 ui toolkit").muted());
    ui.add_child(&root, Spacer::vertical(4.0));

    let card = ui.add_child(&root, VBoxLayout::new());

    let counter = Rc::new(Cell::new(0));
    let counter_label = ui
        .add_child(
            &card,
            Label::new("0")
                .font_size(72.0)
                .bold()
                .align(TextAlign::Center),
        )
        .cast::<Label>()
        .expect("counter label is a Label");

    let status = ui
        .add_child(&card, Label::new("Waiting for input").muted())
        .cast::<Label>()
        .expect("status label is a Label");

    let buttons = ui.add_child(&card, HBoxLayout::new());
    let increment_count = counter.clone();
    let increment_label = counter_label.clone();
    let increment_status = status.clone();
    ui.add_child(
        &buttons,
        Button::new("Increment")
            .primary()
            .min_width(100.0)
            .on_click(move |_ctx| {
                let value = increment_count.get() + 1;
                increment_count.set(value);
                increment_label.edit(|label: &mut Label| {
                    label.set_text(value.to_string());
                });
                increment_status.edit(|label: &mut Label| {
                    label.set_text(format!("Incremented to {value}"));
                });
            }),
    );

    let decrement_count = counter.clone();
    let decrement_label = counter_label.clone();
    let decrement_status = status.clone();
    ui.add_child(
        &buttons,
        Button::new("Decrement")
            .min_width(90.0)
            .on_click(move |_ctx| {
                let value = decrement_count.get() - 1;
                decrement_count.set(value);
                decrement_label.edit(|label: &mut Label| {
                    label.set_text(value.to_string());
                });
                decrement_status.edit(|label: &mut Label| {
                    label.set_text(format!("Decremented to {value}"));
                });
            }),
    );

    let repeat_count = counter.clone();
    let repeat_label = counter_label.clone();
    let repeat_status = status.clone();
    ui.add_child(
        &buttons,
        Button::new("Auto repeat")
            .min_width(110.0)
            .auto_repeat(true)
            .auto_repeat_delay(0.4)
            .auto_repeat_interval(0.15)
            .on_click(move |_ctx| {
                let value = repeat_count.get() + 1;
                repeat_count.set(value);
                repeat_label.edit(|label: &mut Label| {
                    label.set_text(value.to_string());
                });
                repeat_status.edit(|label: &mut Label| {
                    label.set_text(format!("Auto repeated to {value}"));
                });
            }),
    );

    let reset_count = counter.clone();
    let reset_label = counter_label.clone();
    let reset_status = status.clone();
    ui.add_child(
        &buttons,
        Button::new("Reset").min_width(70.0).on_click(move |_ctx| {
            reset_count.set(0);
            reset_label.edit(|label: &mut Label| {
                label.set_text("0");
            });
            reset_status.edit(|label: &mut Label| {
                label.set_text("Reset to 0");
            });
        }),
    );

    let theme_ui = ui.clone();
    let theme_card = card.clone();
    let theme_ui_radio = theme_ui.clone();
    let theme_card_radio = theme_card.clone();
    let layout_ui = ui.clone();
    ui.add_child(
        &buttons,
        Checkbox::new("Debug layout").on_click(move |_ctx| {
            layout_ui.set_layout_debug(!layout_ui.layout_debug());
        }),
    );

    // Beatles tab: the album picker and the song filter.
    let beatles = ui.add_child(&tabs.handle(), VBoxLayout::new());
    ui.add_child(&beatles, Label::new("Beatles album").muted().font_size(14.0));
    let combo_handle = ui
        .add_child(
            &beatles,
            ComboBox::new(beatles::ALBUMS)
                .selected(0)
                .placeholder("Choose a Beatles album"),
        )
        .cast::<ComboBox>()
        .expect("combo box");

    ui.add_child(&beatles, Label::new("Search songs").muted().font_size(14.0));
    let search_handle = ui
        .add_child(
            &beatles,
            Input::with_text("").placeholder("Type to filter songs"),
        )
        .cast::<Input>()
        .expect("search input");

    let list_handle = ui
        .add_child(
            &beatles,
            ListView::new(beatles::SONGS[0])
                .min_width(260.0)
                .alternating(true),
        )
        .cast::<ListView>()
        .expect("list view");

    // The search box filters the songs of the selected album. Both controls
    // share the album and the filter text, so picking another album keeps the
    // filter applied.
    let album = Rc::new(Cell::new(0usize));
    let query = Rc::new(std::cell::RefCell::new(String::new()));

    let album_for_combo = album.clone();
    let query_for_combo = query.clone();
    let list_for_combo = list_handle.clone();
    combo_handle.edit(|combo: &mut ComboBox| {
        combo.set_on_change(move |index, _ctx| {
            album_for_combo.set(index);
            let songs = beatles::SONGS.get(index).copied().unwrap_or(&[]);
            list_for_combo.edit(|list: &mut ListView| {
                // The filter is kept when the album changes.
                list.set_items(songs);
                list.set_filter(query_for_combo.borrow().as_str());
            });
        });
    });

    let query_for_input = query.clone();
    let list_for_input = list_handle.clone();
    search_handle.edit(|input: &mut Input| {
        input.set_on_change(move |text, _ctx| {
            *query_for_input.borrow_mut() = text.to_string();
            list_for_input.edit(|list: &mut ListView| {
                list.set_filter(text);
            });
        });
    });

    // Radio button group: Windows theme presets.
    let group = ui.add_child(
        &root,
        RadioButtonGroup::new().on_change(move |id| {
            let next = match id.as_str() {
                "windows-light" => Theme::light(),
                "windows-dark" => Theme::dark(),
                "windows95-light" => Theme::windows95_light(),
                "windows95-dark" => Theme::windows95(),
                "windows-11" => Theme::windows11(),
                "material" => Theme::dark(),
                _ => Theme::light(),
            };
            let surface = next.palette().surface;
            let theme_ui = theme_ui_radio.clone();
            //let theme_card = theme_card_radio.clone();
            theme_ui.set_theme(next);
            /*            theme_card.edit(|card: &mut Column| {
                            card.set_background(surface);
                        });
            */
        }),
    );

    ui.add_child(
        &group,
        RadioButton::new("Windows light").id("windows-light"),
    );
    ui.add_child(&group, RadioButton::new("Windows dark").id("windows-dark"));
    ui.add_child(
        &group,
        RadioButton::new("Windows 95 light").id("windows95-light"),
    );
    ui.add_child(
        &group,
        RadioButton::new("Windows 95 dark").id("windows95-dark"),
    );
    ui.add_child(&group, RadioButton::new("Material").id("material"));
}
