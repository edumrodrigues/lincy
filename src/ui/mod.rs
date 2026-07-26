use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gtk4::gdk::Key;
use gtk4::glib::{self, Propagation};
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Entry, Label, ListBox, ListBoxRow, Orientation, Picture, Window};

use crate::clipboard::ClipboardManager;
use crate::db::{models::ClipboardItem, Database};

const FULL_TEXT_KEY: &str = "lincy-full-text";
const ITEM_TYPE_KEY: &str = "lincy-item-type";
const IMG_DATA_KEY: &str = "lincy-img-data";
const IMG_WIDTH_KEY: &str = "lincy-img-width";
const IMG_HEIGHT_KEY: &str = "lincy-img-height";

const THUMB_SIZE: i32 = 22;

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
            .application(app).title("Lincy")
            .default_width(460).default_height(400)
            .resizable(true).decorated(false)
            .build();

        let css = gtk4::CssProvider::new();
        css.load_from_string(
            "
            window { border-radius:12px; border:1px solid @borders; }
            entry { margin:8px; border-radius:8px; font-size:14px; }
            list { background-color:@theme_bg_color; color:@theme_fg_color; }
            list row { padding:4px 8px; border-bottom:1px solid alpha(@borders,0.15); color:@theme_fg_color; }
            list row.current-clipboard { background-color:alpha(@theme_selected_bg_color,0.12); }
            list row.current-clipboard:hover { background-color:alpha(@theme_selected_bg_color,0.35); }
            list row:selected { background-color:@theme_selected_bg_color; color:@theme_selected_fg_color; font-weight:bold; }
            list row:hover { background-color:alpha(@theme_selected_bg_color,0.3); }
            label.status { font-size:11px; opacity:0.7; padding:4px 10px; }
            label.item-label { font-size:13px; color:inherit; }
            label.img-label { font-size:11px; opacity:0.7; }
            picture.thumb { border-radius:4px; margin-right:4px; }
            ",
        );
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(), &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let vbox = GtkBox::new(Orientation::Vertical, 0);
        window.set_child(Some(&vbox));

        let search_entry = Entry::builder()
            .placeholder_text("Search clipboard history...").hexpand(true).build();
        vbox.append(&search_entry);
        vbox.append(&gtk4::Separator::new(Orientation::Horizontal));

        let scrolled = gtk4::ScrolledWindow::builder()
            .vexpand(true).hexpand(true)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .build();
        let list_box = ListBox::new();
        list_box.set_vexpand(true);
        scrolled.set_child(Some(&list_box));
        vbox.append(&scrolled);

        let footer = GtkBox::new(Orientation::Horizontal, 0);
        let status_label = Label::new(Some(""));
        status_label.add_css_class("status");
        status_label.set_halign(Align::Start); status_label.set_hexpand(true);
        footer.append(&status_label);
        let hints = Label::new(Some("↩ Copy  Click=Copy  ⎋ Close  ⌫ Delete  ⌃P Pin"));
        hints.add_css_class("status"); hints.set_halign(Align::End);
        footer.append(&hints);
        vbox.append(&footer);

        // Search filtering
        let lbs = list_box.clone(); let ss = status_label.clone();
        let dbs = db.clone(); let cs = clip.clone();
        search_entry.connect_changed(move |e| {
            refresh_list(&lbs, &ss, &dbs, &cs, &e.text());
        });

        // Keyboard shortcuts
        let kc = gtk4::EventControllerKey::new();
        let wk = window.clone(); let lk = list_box.clone();
        let ck = clip.clone(); let dk = db.clone();
        let sk = search_entry.clone(); let stk = status_label.clone();
        kc.connect_key_pressed(move |_, kv, _, mods| {
            match kv {
                Key::Escape => { wk.set_visible(false); return Propagation::Stop; }
                Key::Return | Key::KP_Enter => {
                    copy_selected(&lk, &ck, &stk, &dk);
                    wk.set_visible(false);
                    return Propagation::Stop;
                }
                Key::Delete | Key::KP_Delete => {
                    delete_selected(&lk, &dk, &stk);
                    return Propagation::Stop;
                }
                Key::p | Key::P if mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK) => {
                    toggle_pin_selected(&lk, &dk, &stk);
                    return Propagation::Stop;
                }
                _ => {}
            }
            Propagation::Proceed
        });
        window.add_controller(kc);

        unsafe { list_box.set_data("lincy-window", window.clone()); }

        // Double-click/Space → copy
        let la = list_box.clone(); let ca = clip.clone();
        let sa = status_label.clone(); let da = db.clone(); let wa = window.clone();
        list_box.connect_row_activated(move |_, _| {
            copy_selected(&la, &ca, &sa, &da);
            wa.set_visible(false);
        });

        window.connect_close_request(|w| { w.set_visible(false); Propagation::Stop });
        window.connect_is_active_notify(|w| { if !w.is_active() { w.set_visible(false); } });

        PopupWindow { window, search_entry, list_box, status_label, db, clip, refresh_timer: RefCell::new(None) }
    }

    pub fn show(&self) {
        self.search_entry.set_text("");
        self.window.present();
        self.search_entry.grab_focus();
        refresh_list(&self.list_box, &self.status_label, &self.db, &self.clip, "");

        if self.refresh_timer.borrow().is_none() {
            let l = self.list_box.clone(); let s = self.status_label.clone();
            let d = self.db.clone(); let c = self.clip.clone();
            let mut last_count: i64 = -1;
            let id = glib::timeout_add_local(Duration::from_millis(800), move || {
                if !l.is_visible() { return glib::ControlFlow::Continue; }
                if let Ok(cur) = d.count() {
                    if cur != last_count { last_count = cur; refresh_list(&l, &s, &d, &c, ""); }
                }
                glib::ControlFlow::Continue
            });
            *self.refresh_timer.borrow_mut() = Some(id);
        }
    }

    pub fn hide(&self) { self.window.set_visible(false); }
    pub fn toggle(&self) { if self.window.is_visible() { self.hide(); } else { self.show(); } }
    pub fn is_visible(&self) -> bool { self.window.is_visible() }
}

// ── refresh_list ──────────────────────────────────────────────────────────

fn refresh_list(
    list_box: &ListBox, status_label: &Label,
    db: &Arc<Database>, clip: &Arc<ClipboardManager>, query: &str,
) {
    let result = if query.is_empty() { db.list_recent(100) } else { db.search(query, 100) };

    let selected_text: Option<String> = list_box.selected_row()
        .and_then(|r| unsafe { r.data::<String>(FULL_TEXT_KEY).map(|p| p.as_ref().clone()) });

    while let Some(c) = list_box.first_child() { list_box.remove(&c); }

    match result {
        Ok(items) => {
            if items.is_empty() {
                let row = ListBoxRow::new();
                let l = Label::new(Some(if query.is_empty() { "Clipboard empty. Start copying!" } else { "No matching items." }));
                l.set_halign(Align::Center); l.set_opacity(0.5);
                l.set_margin_top(20); l.set_margin_bottom(20);
                row.set_child(Some(&l)); row.set_selectable(false); row.set_activatable(false);
                list_box.append(&row);
            } else {
                let mut row_to_select: Option<ListBoxRow> = None;
                for (index, item) in items.iter().enumerate() {
                    let row = ListBoxRow::new();
                    let hbox = GtkBox::new(Orientation::Horizontal, 8);
                    hbox.set_margin_top(2); hbox.set_margin_bottom(2);

                    if item.content_type == "image" {
                        build_image_row(&hbox, item);
                    } else {
                        build_text_row(&hbox, item);
                    }

                    row.set_child(Some(&hbox));
                    if index == 0 { row.add_css_class("current-clipboard"); }

                    // Store item data on row
                    unsafe {
                        row.set_data(FULL_TEXT_KEY, item.content.clone());
                        row.set_data(ITEM_TYPE_KEY, item.content_type.clone());
                        if let (Some(d), Some(w), Some(h)) = (&item.image_data, item.image_width, item.image_height) {
                            row.set_data(IMG_DATA_KEY, d.clone());
                            row.set_data(IMG_WIDTH_KEY, w);
                            row.set_data(IMG_HEIGHT_KEY, h);
                        }
                    }

                    // Click → copy + close
                    let click = gtk4::GestureClick::new();
                    let clip_click = Arc::clone(clip);
                    let db_click = Arc::clone(db);
                    let item_click = item.clone();
                    let list_click = list_box.clone();
                    click.connect_pressed(move |g, _, _, _| {
                        copy_item(&clip_click, &db_click, &item_click);
                        unsafe {
                            if let Some(p) = list_click.data::<Window>("lincy-window") {
                                p.as_ref().set_visible(false);
                            }
                        }
                        g.set_state(gtk4::EventSequenceState::Claimed);
                    });
                    row.add_controller(click);

                    if let Some(ref sel) = selected_text { if *sel == item.content { row_to_select = Some(row.clone()); } }
                    list_box.append(&row);
                }
                if let Some(r) = row_to_select { list_box.select_row(Some(&r)); }
                else if let Some(f) = list_box.first_child() { list_box.select_row(Some(&f.downcast::<ListBoxRow>().unwrap())); }
            }
            status_label.set_text(&format!("{} items", items.len()));
        }
        Err(e) => { log::error!("Failed load: {:?}", e); status_label.set_text("Error"); }
    }
}

fn build_text_row(hbox: &GtkBox, item: &ClipboardItem) {
    let trimmed = item.content.trim_start();
    let first = if trimmed.is_empty() { "(empty)" } else { trimmed.lines().next().unwrap_or(trimmed) };
    let display: String = if first.len() > 100 { format!("{}…", &first[..100]) } else { first.to_string() };
    let display = display.replace('\n', " ").replace('\t', " ");
    let prefix = if item.pinned { "📌 " } else { "" };
    let label = Label::new(Some(&format!("{}{}", prefix, display)));
    label.set_halign(Align::Start); label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label.set_xalign(0.0); label.add_css_class("item-label"); label.set_valign(Align::Center);
    hbox.append(&label);
}

fn build_image_row(hbox: &GtkBox, item: &ClipboardItem) {
    let prefix = if item.pinned { "📌 " } else { "" };

    if let (Some(data), Some(w), Some(h)) = (&item.image_data, item.image_width, item.image_height) {
        if w > 0 && h > 0 {
            let bytes = glib::Bytes::from(data.as_slice());
            let texture = gdk4::MemoryTexture::new(
                w, h, gdk4::MemoryFormat::R8g8b8a8, &bytes, (w * 4) as usize,
            );
            let pic = Picture::for_paintable(&texture);
            pic.set_can_shrink(true);
            pic.set_size_request(THUMB_SIZE, THUMB_SIZE);
            pic.add_css_class("thumb");
            hbox.append(&pic);
        }
    }

    let label = Label::new(Some(&format!(
        "{}🖼 Image ({}×{})", prefix,
        item.image_width.unwrap_or(0), item.image_height.unwrap_or(0)
    )));
    label.set_halign(Align::Start); label.set_valign(Align::Center);
    label.add_css_class("img-label");
    hbox.append(&label);
}

// ── Actions ───────────────────────────────────────────────────────────────

fn copy_item(clip: &ClipboardManager, db: &Database, item: &ClipboardItem) {
    if item.content_type == "image" {
        if let (Some(data), Some(w), Some(h)) = (&item.image_data, item.image_width, item.image_height) {
            clip.set_image(data, w, h);
            log::info!("Copied image: {}x{}", w, h);
        }
    } else {
        clip.set_text(&item.content);
        let _ = db.insert_or_update(&item.content);
        log::info!("Copied text: {} chars", item.content.len());
    }
}

fn copy_selected(list_box: &ListBox, clipboard: &ClipboardManager, status: &Label, db: &Database) {
    if let Some(row) = list_box.selected_row() {
        unsafe {
            let item_type = row.data::<String>(ITEM_TYPE_KEY).map(|p| p.as_ref().clone());
            if let Some(t) = item_type {
                if t == "image" {
                    if let (Some(d), Some(w), Some(h)) = (
                        row.data::<Vec<u8>>(IMG_DATA_KEY).map(|p| p.as_ref().clone()),
                        row.data::<i32>(IMG_WIDTH_KEY).map(|p| *p.as_ref()),
                        row.data::<i32>(IMG_HEIGHT_KEY).map(|p| *p.as_ref()),
                    ) {
                        clipboard.set_image(&d, w, h);
                        status.set_text(&format!("Copied image: {}×{}", w, h));
                        return;
                    }
                }
            }
            if let Some(p) = row.data::<String>(FULL_TEXT_KEY) {
                let text = p.as_ref();
                clipboard.set_text(text);
                let _ = db.insert_or_update(text);
                status.set_text(&format!("Copied: {}…", &text[..text.len().min(40)]));
            }
        }
    }
}

fn delete_selected(list_box: &ListBox, db: &Database, status: &Label) {
    let text = list_box.selected_row().and_then(|r| unsafe {
        r.data::<String>(FULL_TEXT_KEY).map(|p| p.as_ref().clone())
    });
    if let Some(t) = text {
        if let Ok(items) = db.list_recent(1000) {
            for item in &items { if item.content == t { let _ = db.delete_item(item.id); status.set_text("Deleted"); break; } }
        }
    }
}

fn toggle_pin_selected(list_box: &ListBox, db: &Database, status: &Label) {
    let text = list_box.selected_row().and_then(|r| unsafe {
        r.data::<String>(FULL_TEXT_KEY).map(|p| p.as_ref().clone())
    });
    if let Some(t) = text {
        if let Ok(items) = db.list_recent(1000) {
            for item in &items {
                if item.content == t { let _ = db.toggle_pin(item.id); status.set_text(if item.pinned {"Unpinned"} else {"Pinned"}); break; }
            }
        }
    }
}
