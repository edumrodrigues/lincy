mod clipboard;
mod config;
mod db;
mod error;
mod hotkey;
mod tray;
mod ui;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use gtk4::gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::glib;

use tray::TrayAction;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let settings = Rc::new(RefCell::new(config::Settings::load()));

    let app = gtk4::Application::builder()
        .application_id(config::app_id())
        .flags(gtk4::gio::ApplicationFlags::default())
        .build();

    let initialized = Arc::new(AtomicBool::new(false));
    let popup_holder: Rc<RefCell<Option<Rc<ui::PopupWindow>>>> = Rc::new(RefCell::new(None));
    let hold_guard: Rc<RefCell<Option<gtk4::gio::ApplicationHoldGuard>>> = Rc::new(RefCell::new(None));
    let poll_guard_holder: Rc<RefCell<Option<clipboard::PollGuard>>> = Rc::new(RefCell::new(None));

    app.connect_activate({
        let initialized = initialized.clone();
        let popup_holder = popup_holder.clone();
        let hold_guard = hold_guard.clone();
        let poll_guard_holder = poll_guard_holder.clone();
        let settings = settings.clone();

        move |app| {
            if initialized.swap(true, Ordering::SeqCst) {
                log::info!("Already running, showing window");
                if let Some(ref p) = *popup_holder.borrow() { p.show(); }
                return;
            }

            log::info!("Starting Lincy");

            let db = match db::Database::new() {
                Ok(db) => { log::info!("DB ready ({} items)", db.count().unwrap_or(0)); Arc::new(db) }
                Err(e) => { log::error!("DB: {}", e); return; }
            };

            let clip = match clipboard::ClipboardManager::new() {
                Ok(c) => Arc::new(c),
                Err(e) => { log::error!("Clipboard: {}", e); return; }
            };

            // Shared thumb_size so settings changes apply immediately
            let thumb_size = Rc::new(RefCell::new(settings.borrow().thumb_size));
            let popup = Rc::new(ui::PopupWindow::new(app, db.clone(), clip.clone(), thumb_size.clone()));
            *popup_holder.borrow_mut() = Some(popup.clone());
            *poll_guard_holder.borrow_mut() = Some(clip.start_monitoring(db.clone(), 500));

            // Tray — spawn via blocking API (synchronous, no tokio needed)
            let (lincy_tray, tray_rx, tray_entries) = tray::LincyTray::new();
            let tray_handle = lincy_tray.spawn_blocking();
            log::info!("Tray icon ready");

            // Hotkey — spawn tokio thread just for ashpd
            let hotkey_rx = {
                let (tx, rx) = mpsc::channel();
                let sc = settings.borrow().shortcut.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio");
                    let ht = tx.clone();
                    rt.spawn(async move {
                        if let Err(e) = hotkey::try_register(&sc, ht).await { log::info!("Hotkey: {}", e); }
                    });
                    rt.block_on(std::future::pending::<()>());
                });
                rx
            };

            *hold_guard.borrow_mut() = Some(app.hold());

            // Main timer
            let popup_t = popup.clone();
            let db_t = db.clone();
            let clip_t = clip.clone();
            let app_t = app.clone();
            let hg_t = hold_guard.clone();
            let tray_entries_t = tray_entries.clone();
            let settings_t = settings.clone();
            let popup_holder_t = popup_holder.clone();

            glib::timeout_add_local(Duration::from_millis(500), move || {
                // Update tray entries + trigger menu refresh
                if let Ok(items) = db_t.list_recent(15) {
                    let entries: Vec<tray::TrayEntry> = items.iter().map(|i| tray::TrayEntry {
                        content: i.content.clone(), content_type: i.content_type.clone(),
                        pinned: i.pinned, image_data: i.image_data.clone(),
                        image_width: i.image_width, image_height: i.image_height,
                    }).collect();
                    // Update shared state
                    if let Ok(mut e) = tray_entries_t.lock() { *e = entries.clone(); }
                    // Trigger ksni menu refresh (blocking, fast)
                    tray::LincyTray::update_menu(&tray_handle, entries, 15);
                }

                // Enforce max items
                let max = settings_t.borrow().max_items;
                if let Ok(count) = db_t.count() && count > max
                    && let Ok(items) = db_t.list_recent(max + 100)
                {
                    for item in items.iter().skip(max as usize) {
                        if !item.pinned { let _ = db_t.delete_item(item.id); }
                    }
                }

                // Tray actions
                while let Ok(a) = tray_rx.try_recv() {
                    match a {
                        TrayAction::Show => popup_t.show(),
                        TrayAction::Quit => {
                            settings_t.borrow().save();
                            popup_t.hide(); *hg_t.borrow_mut() = None; app_t.quit();
                            return glib::ControlFlow::Break;
                        }
                        TrayAction::Settings => {
                            if let Some(ref p) = *popup_holder_t.borrow() {
                                let cur = settings_t.borrow().clone();
                                match ui::settings::show_settings(p.window(), &cur) {
                                    ui::settings::SettingsResult::Save(s) => {
                                        *thumb_size.borrow_mut() = s.thumb_size;
                                        *settings_t.borrow_mut() = s.clone();
                                        s.save();
                                    }
                                    ui::settings::SettingsResult::ClearHistory => {
                                        let _ = db_t.delete_all_unpinned();
                                    }
                                    ui::settings::SettingsResult::Cancel => {}
                                }
                            }
                        }
                        TrayAction::CopyText(t) => {
                            clip_t.set_text(&t); let _ = db_t.insert_or_update(&t);
                        }
                        TrayAction::CopyImage { rgba, width, height } => {
                            clip_t.set_image(&rgba, width, height);
                        }
                    }
                }

                while let Ok(()) = hotkey_rx.try_recv() { popup_t.toggle(); }
                glib::ControlFlow::Continue
            });

            log::info!("Lincy running — {} or tray icon", settings.borrow().shortcut);
            eprintln!("\n  \x1b[1;32m🄋 Lincy is running!\x1b[0m\n\
                         \x1b[90m  {} → popup | Tray → history | ⚙ Settings\x1b[0m\n",
                      settings.borrow().shortcut);
        }
    });

    app.run();
}
