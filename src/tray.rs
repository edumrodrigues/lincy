use std::sync::{Arc, Mutex, mpsc};

use ksni::blocking::{Handle, TrayMethods};
use ksni::menu::{MenuItem, StandardItem};
use ksni::{ToolTip, Tray};

#[derive(Debug, Clone)]
pub enum TrayAction {
    Show,
    Quit,
    Settings,
    CopyText(String),
    CopyImage { rgba: Vec<u8>, width: i32, height: i32 },
}

#[derive(Debug, Clone)]
pub struct TrayEntry {
    pub content: String,
    pub content_type: String,
    pub pinned: bool,
    pub image_data: Option<Vec<u8>>,
    pub image_width: Option<i32>,
    pub image_height: Option<i32>,
}

pub struct LincyTray {
    sender: mpsc::Sender<TrayAction>,
    entries: Arc<Mutex<Vec<TrayEntry>>>,
    max_menu: usize,
}

impl LincyTray {
    pub fn new() -> (Self, mpsc::Receiver<TrayAction>, Arc<Mutex<Vec<TrayEntry>>>) {
        let (tx, rx) = mpsc::channel();
        let entries = Arc::new(Mutex::new(Vec::new()));
        (Self { sender: tx, entries: entries.clone(), max_menu: 15 }, rx, entries)
    }

    /// Spawn the tray service synchronously (blocking API).
    pub fn spawn_blocking(self) -> Handle<Self> {
        self.spawn().expect("Failed to spawn tray service")
    }

    /// Trigger a menu refresh. Safe to call from GTK main loop.
    pub fn update_menu(handle: &Handle<Self>, new_entries: Vec<TrayEntry>, max_menu: usize) {
        handle.update(|tray| {
            *tray.entries.lock().unwrap() = new_entries;
            tray.max_menu = max_menu;
        });
    }
}

impl Tray for LincyTray {
    fn id(&self) -> String { "com.github.edumrodrigues.Lincy".into() }
    fn icon_name(&self) -> String { "lincy".into() }
    fn title(&self) -> String { "Lincy".into() }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.send(TrayAction::Show);
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip { title: "Lincy".into(), description: "Clipboard Manager".into(), icon_name: "lincy".into(), icon_pixmap: vec![] }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut items: Vec<MenuItem<Self>> = Vec::new();
        let entries = self.entries.lock().unwrap();

        for entry in entries.iter().take(self.max_menu) {
            if entry.content_type == "image" {
                let w = entry.image_width.unwrap_or(0);
                let h = entry.image_height.unwrap_or(0);
                let prefix = if entry.pinned { "📌 " } else { "" };
                let label = format!("{}🖼 Image ({}×{})", prefix, w, h);
                let img_data = entry.image_data.clone();
                let img_w = entry.image_width;
                let img_h = entry.image_height;
                let tx = self.sender.clone();
                items.push(MenuItem::Standard(StandardItem {
                    label, enabled: true, visible: true,
                    icon_name: String::new(), icon_data: vec![], shortcut: vec![],
                    disposition: ksni::menu::Disposition::Normal,
                    activate: Box::new(move |_| {
                        if let (Some(d), Some(w), Some(h)) = (&img_data, img_w, img_h) {
                            let _ = tx.send(TrayAction::CopyImage { rgba: d.clone(), width: w, height: h });
                        }
                    }),
                }));
            } else {
                let trimmed = entry.content.trim_start();
                let first = if trimmed.is_empty() { "(empty)" } else { trimmed.lines().next().unwrap_or(trimmed) };
                let display = if first.len() > 50 { format!("{}…", &first[..50]) } else { first.to_string() };
                let prefix = if entry.pinned { "📌 " } else { "" };
                let label = format!("{}{}", prefix, display);
                let text = entry.content.clone();
                let tx = self.sender.clone();
                items.push(MenuItem::Standard(StandardItem {
                    label, enabled: true, visible: true,
                    icon_name: String::new(), icon_data: vec![], shortcut: vec![],
                    disposition: ksni::menu::Disposition::Normal,
                    activate: Box::new(move |_| { let _ = tx.send(TrayAction::CopyText(text.clone())); }),
                }));
            }
        }

        if !items.is_empty() { items.push(MenuItem::Separator); }

        let settings_tx = self.sender.clone();
        items.push(MenuItem::Standard(StandardItem {
            label: "⚙ Settings…".into(), enabled: true, visible: true,
            icon_name: "emblem-system-symbolic".into(), icon_data: vec![], shortcut: vec![],
            disposition: ksni::menu::Disposition::Normal,
            activate: Box::new(move |_| { let _ = settings_tx.send(TrayAction::Settings); }),
        }));

        let quit_tx = self.sender.clone();
        items.push(MenuItem::Standard(StandardItem {
            label: "Quit".into(), enabled: true, visible: true,
            icon_name: "application-exit-symbolic".into(), icon_data: vec![], shortcut: vec![],
            disposition: ksni::menu::Disposition::Normal,
            activate: Box::new(move |_| { let _ = quit_tx.send(TrayAction::Quit); }),
        }));

        items
    }

    fn category(&self) -> ksni::Category { ksni::Category::ApplicationStatus }
    fn status(&self) -> ksni::Status { ksni::Status::Active }
}
