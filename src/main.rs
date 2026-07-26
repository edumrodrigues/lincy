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
use std::sync::mpsc;
use std::sync::Arc;

use gtk4::gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::glib;

use tray::TrayAction;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let app = gtk4::Application::builder()
        .application_id(config::app_id())
        .flags(gtk4::gio::ApplicationFlags::default())
        .build();

    let initialized = Arc::new(AtomicBool::new(false));
    let popup_holder: Rc<RefCell<Option<Rc<ui::PopupWindow>>>> = Rc::new(RefCell::new(None));
    let hold_guard: Rc<RefCell<Option<gtk4::gio::ApplicationHoldGuard>>> = Rc::new(RefCell::new(None));
    // Keep the poll guard alive for the lifetime of the app
    let poll_guard_holder: Rc<RefCell<Option<clipboard::PollGuard>>> = Rc::new(RefCell::new(None));

    app.connect_activate({
        let initialized = initialized.clone();
        let popup_holder = popup_holder.clone();
        let hold_guard = hold_guard.clone();
        let poll_guard_holder = poll_guard_holder.clone();

        move |app| {
            if initialized.swap(true, Ordering::SeqCst) {
                log::info!("Lincy already running, showing window");
                if let Some(ref popup) = *popup_holder.borrow() {
                    popup.show();
                }
                return;
            }

            log::info!("Starting Lincy clipboard manager");

            // ── Database ──────────────────────────────────────────
            let db = match db::Database::new() {
                Ok(db) => {
                    log::info!(
                        "Database ready at {} ({} items)",
                        config::db_path().display(),
                        db.count().unwrap_or(0)
                    );
                    Arc::new(db)
                }
                Err(e) => {
                    log::error!("Failed to open database: {}", e);
                    return;
                }
            };

            // ── Clipboard (arboard, works via XWayland) ───────────
            let clip = match clipboard::ClipboardManager::new() {
                Ok(c) => Arc::new(c),
                Err(e) => {
                    log::error!("Failed to initialize clipboard: {}", e);
                    return;
                }
            };

            // ── Popup window ─────────────────────────────────────
            let popup = Rc::new(ui::PopupWindow::new(app, db.clone(), clip.clone()));
            *popup_holder.borrow_mut() = Some(popup.clone());

            // ── Clipboard polling (500ms) ────────────────────────
            *poll_guard_holder.borrow_mut() = Some(clip.start_monitoring(db.clone(), 500));

            // ── Tray icon (ksni / D-Bus StatusNotifierItem) ──────
            let (lincy_tray, tray_rx) = tray::LincyTray::new();

            // Background thread: tokio for tray + hotkey
            let hotkey_rx = {
                let (tx, rx) = mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("tokio runtime");

                    lincy_tray.spawn_on_runtime(&rt);

                    let hotkey_tx = tx.clone();
                    rt.spawn(async move {
                        if let Err(e) =
                            hotkey::try_register("Ctrl+Shift+C", hotkey_tx).await
                        {
                            log::info!(
                                "Portal hotkey not available: {}. Using GNOME custom shortcut.",
                                e
                            );
                        }
                    });

                    rt.block_on(std::future::pending::<()>());
                });
                rx
            };

            // ── Keep app alive when window is hidden ─────────────
            let guard = app.hold();
            *hold_guard.borrow_mut() = Some(guard);

            // ── Timer: poll tray events & hotkey ──────────────────
            let popup_timer = popup.clone();
            let db_timer = db.clone();
            let app_timer = app.clone();
            let hold_guard_timer = hold_guard.clone();

            glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
                // Tray menu actions
                while let Ok(action) = tray_rx.try_recv() {
                    match action {
                        TrayAction::Show => popup_timer.show(),
                        TrayAction::ClearHistory => {
                            let _ = db_timer.delete_all_unpinned();
                            if popup_timer.is_visible() {
                                popup_timer.show();
                            }
                        }
                        TrayAction::Quit => {
                            log::info!("Quitting Lincy");
                            popup_timer.hide();
                            *hold_guard_timer.borrow_mut() = None;
                            app_timer.quit();
                            return glib::ControlFlow::Break;
                        }
                    }
                }

                // Hotkey activations
                while let Ok(()) = hotkey_rx.try_recv() {
                    popup_timer.toggle();
                }

                glib::ControlFlow::Continue
            });

            log::info!("Lincy is running — copy text anywhere to capture it");
            eprintln!(
                "\n  \x1b[1;32m🄋 Lincy is running!\x1b[0m\n\
                   \x1b[90m  Copy text → stored in history.\n\
                   \x1b[90m  Ctrl+Shift+C → popup | Tray icon → menu | 'lincy' → popup\x1b[0m\n"
            );
        }
    });

    app.run();
}
