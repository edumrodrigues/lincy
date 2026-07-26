use std::sync::mpsc;

use ksni::menu::{MenuItem, StandardItem};
use ksni::{Icon, ToolTip, Tray, TrayMethods};

/// Actions that can be triggered from the tray menu or activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Show,
    ClearHistory,
    Quit,
}

/// The StatusNotifierItem tray implementation. Communicates with the GTK main
/// thread via an mpsc channel.
pub struct LincyTray {
    sender: mpsc::Sender<TrayAction>,
}

impl LincyTray {
    /// Creates a new tray instance and returns it along with the receiver end
    /// for polling in the GTK main loop.
    pub fn new() -> (Self, mpsc::Receiver<TrayAction>) {
        let (tx, rx) = mpsc::channel();
        (Self { sender: tx }, rx)
    }

    /// Spawns the tray service on the provided tokio runtime. This registers
    /// the StatusNotifierItem on D-Bus and keeps running until the runtime
    /// is dropped.
    pub fn spawn_on_runtime(self, rt: &tokio::runtime::Runtime) {
        rt.spawn(async move {
            if let Err(e) = self.spawn().await {
                log::error!("Failed to spawn tray service: {}", e);
            }
        });
    }
}

impl Tray for LincyTray {
    fn id(&self) -> String {
        "com.github.edumrodrigues.Lincy".into()
    }

    fn icon_name(&self) -> String {
        // Use our installed icon; falls back to default if not found
        "lincy".into()
    }

    fn title(&self) -> String {
        "Lincy".into()
    }

    /// Left-click on the tray icon → show the popup.
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.send(TrayAction::Show);
    }

    /// Right-click or middle-click → also show the popup.
    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.send(TrayAction::Show);
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: "Lincy".into(),
            description: "Clipboard Manager".into(),
            icon_name: "lincy".into(),
            icon_pixmap: vec![],
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let show_tx = self.sender.clone();
        let clear_tx = self.sender.clone();
        let quit_tx = self.sender.clone();

        vec![
            MenuItem::Standard(StandardItem {
                label: "Show Lincy".into(),
                enabled: true,
                visible: true,
                icon_name: String::new(),
                icon_data: vec![],
                shortcut: vec![],
                disposition: ksni::menu::Disposition::Normal,
                activate: Box::new(move |_tray: &mut Self| {
                    let _ = show_tx.send(TrayAction::Show);
                }),
            }),
            MenuItem::Standard(StandardItem {
                label: "Clear History".into(),
                enabled: true,
                visible: true,
                icon_name: "edit-clear-all-symbolic".into(),
                icon_data: vec![],
                shortcut: vec![],
                disposition: ksni::menu::Disposition::Normal,
                activate: Box::new(move |_tray: &mut Self| {
                    let _ = clear_tx.send(TrayAction::ClearHistory);
                }),
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Quit".into(),
                enabled: true,
                visible: true,
                icon_name: "application-exit-symbolic".into(),
                icon_data: vec![],
                shortcut: vec![],
                disposition: ksni::menu::Disposition::Normal,
                activate: Box::new(move |_tray: &mut Self| {
                    let _ = quit_tx.send(TrayAction::Quit);
                }),
            }),
        ]
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        // Provide a fallback pixmap in case the named icon isn't found
        vec![make_icon_pixmap()]
    }

    fn category(&self) -> ksni::Category {
        ksni::Category::ApplicationStatus
    }

    fn status(&self) -> ksni::Status {
        ksni::Status::Active
    }
}

/// Creates a simple clipboard icon as 24×24 RGBA pixels for the fallback pixmap.
fn make_icon_pixmap() -> Icon {
    let width: i32 = 24;
    let height: i32 = 24;
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;

            let board = x >= 2 && x < 22 && y >= 5 && y < 23;
            let clip = x >= 8 && x < 16 && y >= 1 && y < 6;
            let outline =
                (x == 2 || x == 21) && y >= 5 && y < 23 || (y == 5 || y == 22) && x >= 2 && x < 22;
            let clip_outline =
                (x == 8 || x == 15) && y >= 1 && y < 6 || (y == 1 || y == 5) && x >= 8 && x < 16;

            // Text lines
            let line1 = y == 9 && x >= 5 && x < 19;
            let line2 = y == 13 && x >= 5 && x < 16;
            let line3 = y == 17 && x >= 5 && x < 18;

            if outline || clip_outline {
                pixels[i] = 180;
                pixels[i + 1] = 180;
                pixels[i + 2] = 180;
                pixels[i + 3] = 255;
            } else if clip {
                pixels[i] = 80;
                pixels[i + 1] = 80;
                pixels[i + 2] = 80;
                pixels[i + 3] = 255;
            } else if board {
                pixels[i] = 50;
                pixels[i + 1] = 50;
                pixels[i + 2] = 50;
                pixels[i + 3] = 255;
            } else if line1 || line2 || line3 {
                pixels[i] = 255;
                pixels[i + 1] = 255;
                pixels[i + 2] = 255;
                pixels[i + 3] = 255;
            }
        }
    }

    Icon {
        width,
        height,
        data: pixels,
    }
}
