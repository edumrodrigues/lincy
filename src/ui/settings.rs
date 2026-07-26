use gtk4::gdk::Key;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Grid, Label, Orientation, SpinButton, Window};
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Settings;

pub enum SettingsResult {
    Save(Settings),
    ClearHistory,
    Cancel,
}

fn keyval_to_name(kv: Key) -> &'static str {
    // Map common keys to readable names
    match kv {
        Key::A => "A", Key::B => "B", Key::C => "C", Key::D => "D", Key::E => "E",
        Key::F => "F", Key::G => "G", Key::H => "H", Key::I => "I", Key::J => "J",
        Key::K => "K", Key::L => "L", Key::M => "M", Key::N => "N", Key::O => "O",
        Key::P => "P", Key::Q => "Q", Key::R => "R", Key::S => "S", Key::T => "T",
        Key::U => "U", Key::V => "V", Key::W => "W", Key::X => "X", Key::Y => "Y",
        Key::Z => "Z",
        Key::_0 => "0", Key::_1 => "1", Key::_2 => "2", Key::_3 => "3", Key::_4 => "4",
        Key::_5 => "5", Key::_6 => "6", Key::_7 => "7", Key::_8 => "8", Key::_9 => "9",
        Key::space => "Space", Key::Tab => "Tab", Key::BackSpace => "Backspace",
        Key::Escape => "Escape", Key::Delete => "Delete", Key::Insert => "Insert",
        Key::Home => "Home", Key::End => "End", Key::Page_Up => "PageUp", Key::Page_Down => "PageDown",
        Key::Up => "Up", Key::Down => "Down", Key::Left => "Left", Key::Right => "Right",
        Key::F1 => "F1", Key::F2 => "F2", Key::F3 => "F3", Key::F4 => "F4",
        Key::F5 => "F5", Key::F6 => "F6", Key::F7 => "F7", Key::F8 => "F8",
        Key::F9 => "F9", Key::F10 => "F10", Key::F11 => "F11", Key::F12 => "F12",
        _ => "?",
    }
}

/// Convert captured key+modifiers to gsettings binding format like "<Control><Shift>c"
fn to_gsettings_binding(mods: gtk4::gdk::ModifierType, keyval: Key) -> String {
    let mut s = String::new();
    if mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK) { s.push_str("<Control>"); }
    if mods.contains(gtk4::gdk::ModifierType::SHIFT_MASK)   { s.push_str("<Shift>"); }
    if mods.contains(gtk4::gdk::ModifierType::ALT_MASK)     { s.push_str("<Alt>"); }
    if mods.contains(gtk4::gdk::ModifierType::SUPER_MASK)   { s.push_str("<Super>"); }
    s.push_str(&keyval_to_name(keyval).to_lowercase());
    s
}

/// Convert key+modifiers to human-readable string like "Ctrl+Shift+C"
fn to_human(mods: gtk4::gdk::ModifierType, keyval: Key) -> String {
    let mut parts = Vec::new();
    if mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK) { parts.push("Ctrl"); }
    if mods.contains(gtk4::gdk::ModifierType::SHIFT_MASK)   { parts.push("Shift"); }
    if mods.contains(gtk4::gdk::ModifierType::ALT_MASK)     { parts.push("Alt"); }
    if mods.contains(gtk4::gdk::ModifierType::SUPER_MASK)   { parts.push("Super"); }
    parts.push(keyval_to_name(keyval));
    parts.join("+")
}

fn register_shortcut(binding: &str) {
    let schema_path = "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/lincy/";
    let result = std::process::Command::new("gsettings")
        .args(["set", schema_path, "binding", binding])
        .output();
    match result {
        Ok(o) if o.status.success() => log::info!("Shortcut registered: {}", binding),
        Ok(o) => log::error!("gsettings failed: {}", String::from_utf8_lossy(&o.stderr)),
        Err(e) => log::error!("gsettings error: {}", e),
    }
}

pub fn show_settings(parent: &Window, current: &Settings) -> SettingsResult {
    let dialog = gtk4::Window::builder()
        .title("Lincy Settings")
        .default_width(380).default_height(280)
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

    // Shortcut capture button
    let shortcut_btn = Button::with_label(&current.shortcut);
    shortcut_btn.set_hexpand(true);
    let captured: Rc<RefCell<Option<(gtk4::gdk::ModifierType, Key)>>> = Rc::new(RefCell::new(None));
    let captured_label = Rc::new(RefCell::new(current.shortcut.clone()));
    let shortcut_binding = Rc::new(RefCell::new(String::new()));

    let btn_label = shortcut_btn.clone();
    let captured_btn = captured.clone();
    let captured_label_btn = captured_label.clone();
    let captured_binding = shortcut_binding.clone();

    // Key controller on the button
    let key_ctrl = gtk4::EventControllerKey::new();
    key_ctrl.connect_key_pressed(move |_ctrl, keyval, _code, mods| {
        if keyval != Key::Control_L && keyval != Key::Control_R
            && keyval != Key::Shift_L && keyval != Key::Shift_R
            && keyval != Key::Alt_L && keyval != Key::Alt_R
            && keyval != Key::Super_L && keyval != Key::Super_R
            && keyval != Key::Meta_L && keyval != Key::Meta_R
        {
            let human = to_human(mods, keyval);
            let binding = to_gsettings_binding(mods, keyval);
            btn_label.set_label(&human);
            *captured_btn.borrow_mut() = Some((mods, keyval));
            *captured_label_btn.borrow_mut() = human;
            *captured_binding.borrow_mut() = binding;
        }
        Propagation::Stop
    });
    shortcut_btn.add_controller(key_ctrl);

    // Click to focus and capture
    shortcut_btn.connect_clicked(|btn| {
        btn.set_label("Press keys…");
        btn.grab_focus();
    });

    grid.attach(&shortcut_btn, 1, 2, 1, 1);

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
    let sl = captured_label.clone();
    let sb = shortcut_binding.clone();
    save_btn.connect_clicked(move |_| {
        let shortcut = sl.borrow().clone();
        let binding = sb.borrow().clone();
        if !binding.is_empty() {
            register_shortcut(&binding);
        }
        *r.borrow_mut() = SettingsResult::Save(Settings {
            thumb_size: thumb_spin.value() as i32,
            max_items: max_spin.value() as i64,
            shortcut,
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

    let mut borrowed = result.borrow_mut();
    std::mem::replace(&mut *borrowed, SettingsResult::Cancel)
}
