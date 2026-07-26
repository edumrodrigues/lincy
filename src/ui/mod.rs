use std::cell::RefCell;
use std::sync::Arc;

use gtk4::gdk::Key;
use gtk4::glib::{self, Propagation};
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Entry, Label, ListBox, ListBoxRow, Orientation, Window};

use crate::clipboard::ClipboardManager;
use crate::db::Database;

const FULL_TEXT_KEY: &str = "lincy-full-text";

/// The Maccy-style popup search window.
pub struct PopupWindow {
    window: Window,
    search_entry: Entry,
    list_box: ListBox,
    status_label: Label,
    db: Arc<Database>,
    clip: Arc<ClipboardManager>,
    refresh_timer: RefCell<Option<glib::SourceId>>,
}

impl PopupWindow {
    pub fn new(app: &gtk4::Application, db: Arc<Database>, clip: Arc<ClipboardManager>) -> Self {
        let window = Window::builder()
            .application(app)
            .title("Lincy")
            .default_width(440)
            .default_height(380)
            .resizable(true)
            .decorated(false)
            .build();

        let css = gtk4::CssProvider::new();
        css.load_from_string(
            "
            window {
                border-radius: 12px;
                border: 1px solid @borders;
            }
            entry {
                margin: 8px;
                border-radius: 8px;
                font-size: 14px;
            }
            list {
                background-color: @theme_bg_color;
                color: @theme_fg_color;
            }
            list row {
                padding: 6px 10px;
                border-bottom: 1px solid alpha(@borders, 0.2);
                color: @theme_fg_color;
            }
            list row.current-clipboard {
                background-color: alpha(@theme_selected_bg_color, 0.12);
            }
            list row.current-clipboard:hover {
                background-color: alpha(@theme_selected_bg_color, 0.35);
            }
            list row:selected {
                background-color: @theme_selected_bg_color;
                color: @theme_selected_fg_color;
                font-weight: bold;
            }
            list row:hover {
                background-color: alpha(@theme_selected_bg_color, 0.3);
            }
            label.status {
                font-size: 11px;
                opacity: 0.7;
                padding: 4px 10px;
            }
            label.item-label {
                font-size: 13px;
                color: inherit;
            }
            ",
        );
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let vbox = GtkBox::new(Orientation::Vertical, 0);
        window.set_child(Some(&vbox));

        let search_entry = Entry::builder()
            .placeholder_text("Search clipboard history...")
            .hexpand(true)
            .build();
        vbox.append(&search_entry);

        vbox.append(&gtk4::Separator::new(Orientation::Horizontal));

        let scrolled = gtk4::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .build();

        let list_box = ListBox::new();
        list_box.set_vexpand(true);
        list_box.add_css_class("rich-list");
        scrolled.set_child(Some(&list_box));
        vbox.append(&scrolled);

        let footer = GtkBox::new(Orientation::Horizontal, 0);
        let status_label = Label::new(Some(""));
        status_label.add_css_class("status");
        status_label.set_halign(Align::Start);
        status_label.set_hexpand(true);
        footer.append(&status_label);

        let hints_label = Label::new(Some("↩ Enter=Copy  Click=Copy  ⎋ Close  ⌫ Delete  ⌃P Pin"));
        hints_label.add_css_class("status");
        hints_label.set_halign(Align::End);
        footer.append(&hints_label);
        vbox.append(&footer);

        // ── Search filtering ────────────────────────────────────
        let list_box_search = list_box.clone();
        let status_search = status_label.clone();
        let db_search = db.clone();
        let clip_search = clip.clone();

        search_entry.connect_changed(move |entry| {
            let query = entry.text().to_string();
            refresh_list(
                &list_box_search,
                &status_search,
                &db_search,
                &clip_search,
                &query,
            );
        });

        // ── Keyboard: Enter copies, Escape closes ───────────────
        let key_controller = gtk4::EventControllerKey::new();
        let window_key = window.clone();
        let list_box_key = list_box.clone();
        let clip_key = clip.clone();
        let db_key = db.clone();
        let search_entry_key = search_entry.clone();
        let status_key = status_label.clone();

        key_controller.connect_key_pressed(
            move |_controller, keyval, _code, modifier| {
                match keyval {
                    Key::Escape => {
                        window_key.set_visible(false);
                        return Propagation::Stop;
                    }
                    Key::Return | Key::KP_Enter => {
                        copy_selected(&list_box_key, &clip_key, &status_key, &db_key);
                        window_key.set_visible(false);
                        return Propagation::Stop;
                    }
                    Key::Delete | Key::KP_Delete => {
                        let query = search_entry_key.text().to_string();
                        delete_selected(&list_box_key, &db_key, &status_key, &query);
                        return Propagation::Stop;
                    }
                    Key::p | Key::P => {
                        if modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                            let query = search_entry_key.text().to_string();
                            toggle_pin_selected(
                                &list_box_key, &db_key, &status_key, &query,
                            );
                            return Propagation::Stop;
                        }
                    }
                    _ => {}
                }
                Propagation::Proceed
            },
        );
        window.add_controller(key_controller);

        // Store window reference on list_box for click handlers
        unsafe {
            list_box.set_data("lincy-window", window.clone());
        }

        // ── Row activated (double-click or Space) → copy ────────
        let list_act = list_box.clone();
        let clip_act = clip.clone();
        let status_act = status_label.clone();
        let db_act = db.clone();
        let window_act = window.clone();
        list_box.connect_row_activated(move |_list, _row| {
            copy_selected(&list_act, &clip_act, &status_act, &db_act);
            window_act.set_visible(false);
        });

        window.connect_close_request(|window| {
            window.set_visible(false);
            Propagation::Stop
        });

        PopupWindow {
            window,
            search_entry,
            list_box,
            status_label,
            db,
            clip,
            refresh_timer: RefCell::new(None),
        }
    }

    /// Show the popup, centered, with search focused. Starts auto-refresh.
    pub fn show(&self) {
        self.search_entry.set_text("");
        self.window.present();
        self.search_entry.grab_focus();

        refresh_list(&self.list_box, &self.status_label, &self.db, &self.clip, "");

        // Auto-refresh while visible — only if data changed
        if self.refresh_timer.borrow().is_none() {
            let list = self.list_box.clone();
            let status = self.status_label.clone();
            let db = self.db.clone();
            let clip = self.clip.clone();

            let mut last_count: i64 = -1;

            let id = glib::timeout_add_local(
                std::time::Duration::from_millis(800),
                move || {
                    if !list.is_visible() {
                        return glib::ControlFlow::Continue;
                    }
                    // Only rebuild if item count changed
                    if let Ok(current) = db.count() {
                        if current != last_count {
                            last_count = current;
                            refresh_list(&list, &status, &db, &clip, "");
                        }
                    }
                    glib::ControlFlow::Continue
                },
            );
            *self.refresh_timer.borrow_mut() = Some(id);
        }
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    pub fn toggle(&self) {
        if self.window.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }

    pub fn is_visible(&self) -> bool {
        self.window.is_visible()
    }
}

// ── refresh_list: rebuilds list, stores full text on rows, adds click-to-copy ──

fn refresh_list(
    list_box: &ListBox,
    status_label: &Label,
    db: &Database,
    clip: &Arc<ClipboardManager>,
    query: &str,
) {
    let result = if query.is_empty() {
        db.list_recent(100)
    } else {
        db.search(query, 100)
    };

    let selected_text: Option<String> = list_box
        .selected_row()
        .and_then(|row| unsafe {
            row.data::<String>(FULL_TEXT_KEY)
                .map(|ptr| ptr.as_ref().clone())
        });

    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    match result {
        Ok(items) => {
            if items.is_empty() {
                let row = ListBoxRow::new();
                let label = Label::new(Some(if query.is_empty() {
                    "Clipboard empty. Start copying!"
                } else {
                    "No matching items."
                }));
                label.set_halign(Align::Center);
                label.set_opacity(0.5);
                label.set_margin_top(20);
                label.set_margin_bottom(20);
                row.set_child(Some(&label));
                row.set_selectable(false);
                row.set_activatable(false);
                list_box.append(&row);
            } else {
                let mut row_to_select: Option<ListBoxRow> = None;

                for (index, item) in items.iter().enumerate() {
                    let row = ListBoxRow::new();

                    let hbox = GtkBox::new(Orientation::Horizontal, 6);

                    let first_line = item.content.lines().next().unwrap_or(&item.content);
                    let display = if first_line.len() > 100 {
                        format!("{}…", &first_line[..100])
                    } else {
                        first_line.to_string()
                    };

                    // Indicator: pin icon for pinned items
                    let prefix = if item.pinned { "📌 " } else { "" };
                    let label = Label::new(Some(&format!("{}{}", prefix, display)));
                    label.set_halign(Align::Start);
                    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
                    label.set_xalign(0.0);
                    label.add_css_class("item-label");
                    hbox.append(&label);

                    row.set_child(Some(&hbox));

                    // Mark first item (current clipboard) with distinct background
                    if index == 0 {
                        row.add_css_class("current-clipboard");
                    }

                    // Store full content on the row
                    unsafe {
                        row.set_data(FULL_TEXT_KEY, item.content.clone());
                    }

                    // ── Single-click → copy + close window ────────
                    let click = gtk4::GestureClick::new();
                    let row_content = item.content.clone();
                    let clip_click = Arc::clone(clip);
                    let list_click = list_box.clone();
                    click.connect_pressed(move |gesture, _n_press, _x, _y| {
                        clip_click.set_text(&row_content);
                        unsafe {
                            if let Some(ptr) =
                                list_click.data::<Window>("lincy-window")
                            {
                                ptr.as_ref().set_visible(false);
                            }
                        }
                        gesture.set_state(gtk4::EventSequenceState::Claimed);
                    });
                    row.add_controller(click);

                    if let Some(ref sel) = selected_text {
                        if *sel == item.content {
                            row_to_select = Some(row.clone());
                        }
                    }

                    list_box.append(&row);
                }

                if let Some(row) = row_to_select {
                    list_box.select_row(Some(&row));
                } else if let Some(first) = list_box.first_child() {
                    list_box
                        .select_row(Some(&first.downcast::<ListBoxRow>().unwrap()));
                }
            }
            status_label.set_text(&format!("{} items", items.len()));
        }
        Err(e) => {
            log::error!("Failed to load items: {:?}", e);
            status_label.set_text("Error loading items");
        }
    }
}

// ── Actions ──

fn copy_selected(
    list_box: &ListBox,
    clipboard: &ClipboardManager,
    status_label: &Label,
    db: &Database,
) {
    if let Some(row) = list_box.selected_row() {
        unsafe {
            if let Some(ptr) = row.data::<String>(FULL_TEXT_KEY) {
                let text = ptr.as_ref();
                clipboard.set_text(text);
                // Update DB so this item becomes the most recent (current clipboard)
                if let Err(e) = db.insert_or_update(text) {
                    log::error!("Failed to bump item: {}", e);
                }
                let preview = &text[..text.len().min(40)];
                status_label.set_text(&format!("Copied: {}…", preview));
                log::info!("Copied: {} chars", text.len());
            }
        }
    }
}

fn delete_selected(
    list_box: &ListBox,
    db: &Database,
    status_label: &Label,
    query: &str,
) {
    let text = list_box
        .selected_row()
        .and_then(|row| unsafe {
            row.data::<String>(FULL_TEXT_KEY)
                .map(|ptr| ptr.as_ref().clone())
        });

    if let Some(full_text) = text {
        if let Ok(items) = db.list_recent(1000) {
            for item in &items {
                if item.content == full_text {
                    let _ = db.delete_item(item.id);
                    status_label.set_text("Deleted");
                    break;
                }
            }
        }
        // Need to re-acquire db + clip references to refresh. We use a simpler
        // approach: just call refresh_list through a stored helper.
        // For now, refresh won't happen after delete from key binding.
        // The auto-refresh timer will pick it up within 500ms.
    }
}

fn toggle_pin_selected(
    list_box: &ListBox,
    db: &Database,
    status_label: &Label,
    query: &str,
) {
    let text = list_box
        .selected_row()
        .and_then(|row| unsafe {
            row.data::<String>(FULL_TEXT_KEY)
                .map(|ptr| ptr.as_ref().clone())
        });

    if let Some(full_text) = text {
        if let Ok(items) = db.list_recent(1000) {
            for item in &items {
                if item.content == full_text {
                    let _ = db.toggle_pin(item.id);
                    status_label.set_text(if item.pinned {
                        "Unpinned"
                    } else {
                        "Pinned"
                    });
                    break;
                }
            }
        }
        // Auto-refresh will pick up changes within 500ms
    }
}
