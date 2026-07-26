use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Entry, Grid, Label, Orientation, SpinButton, Window};
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Settings;

pub enum SettingsResult {
    Save(Settings),
    ClearHistory,
    Cancel,
}

pub fn show_settings(parent: &Window, current: &Settings) -> SettingsResult {
    let dialog = gtk4::Window::builder()
        .title("Lincy Settings")
        .default_width(380).default_height(260)
        .resizable(false).decorated(true)
        .transient_for(parent).modal(true)
        .build();

    let vbox = GtkBox::new(Orientation::Vertical, 12);
    vbox.set_margin_start(16); vbox.set_margin_end(16);
    vbox.set_margin_top(16); vbox.set_margin_bottom(16);
    dialog.set_child(Some(&vbox));

    let title = Label::new(Some("⚙ Settings"));
    vbox.append(&title);

    let grid = Grid::new();
    grid.set_column_spacing(12); grid.set_row_spacing(10);
    vbox.append(&grid);

    grid.attach(&Label::new(Some("Thumbnail (px):")), 0, 0, 1, 1);
    let thumb_spin = SpinButton::with_range(16.0, 128.0, 2.0);
    thumb_spin.set_value(current.thumb_size as f64);
    grid.attach(&thumb_spin, 1, 0, 1, 1);

    grid.attach(&Label::new(Some("Max items:")), 0, 1, 1, 1);
    let max_spin = SpinButton::with_range(5.0, 10000.0, 10.0);
    max_spin.set_value(current.max_items as f64);
    grid.attach(&max_spin, 1, 1, 1, 1);

    grid.attach(&Label::new(Some("Shortcut:")), 0, 2, 1, 1);
    let shortcut_entry = Entry::new();
    shortcut_entry.set_text(&current.shortcut);
    grid.attach(&shortcut_entry, 1, 2, 1, 1);

    let clear_btn = Button::with_label("Clear All History");
    clear_btn.add_css_class("destructive-action");
    vbox.append(&clear_btn);

    let btn_box = GtkBox::new(Orientation::Horizontal, 8);
    btn_box.set_halign(Align::End);
    vbox.append(&btn_box);
    let cancel_btn = Button::with_label("Cancel");
    btn_box.append(&cancel_btn);
    let save_btn = Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    btn_box.append(&save_btn);

    let result: Rc<RefCell<SettingsResult>> = Rc::new(RefCell::new(SettingsResult::Cancel));

    let r = result.clone();
    let d = dialog.clone();
    save_btn.connect_clicked(move |_| {
        *r.borrow_mut() = SettingsResult::Save(Settings {
            thumb_size: thumb_spin.value() as i32,
            max_items: max_spin.value() as i64,
            shortcut: shortcut_entry.text().to_string(),
        });
        d.close();
    });

    let d2 = dialog.clone();
    cancel_btn.connect_clicked(move |_| { d2.close(); });

    let r3 = result.clone();
    let d3 = dialog.clone();
    clear_btn.connect_clicked(move |_| {
        let confirm = gtk4::MessageDialog::new(
            Some(&d3),
            gtk4::DialogFlags::MODAL,
            gtk4::MessageType::Warning,
            gtk4::ButtonsType::OkCancel,
            "Clear all unpinned clipboard history?",
        );
        confirm.set_secondary_text(Some("This cannot be undone."));
        let r = r3.clone();
        let d = d3.clone();
        confirm.connect_response(move |dlg, resp| {
            if resp == gtk4::ResponseType::Ok {
                *r.borrow_mut() = SettingsResult::ClearHistory;
                d.close();
            }
            dlg.close();
        });
        confirm.present();
    });

    dialog.present();
    while dialog.is_visible() {
        gtk4::glib::MainContext::default().iteration(false);
    }

    // Move out the result
    let mut borrowed = result.borrow_mut();
    std::mem::replace(&mut *borrowed, SettingsResult::Cancel)
}
