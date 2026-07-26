use std::sync::mpsc;

use crate::error::LincyError;

/// Tries to register a global shortcut via xdg-desktop-portal (best-effort).
/// Sends `()` on the channel when the shortcut is activated.
///
/// On Wayland/GNOME, the portal works when the app is launched via its
/// .desktop file. When run from a terminal, the portal can't identify the
/// app — set up the shortcut manually in GNOME Settings instead.
pub async fn try_register(shortcut: &str, tx: mpsc::Sender<()>) -> Result<(), LincyError> {
    use ashpd::desktop::global_shortcuts::{BindShortcutsOptions, GlobalShortcuts, NewShortcut};
    use ashpd::desktop::CreateSessionOptions;
    use futures::StreamExt;

    let proxy = GlobalShortcuts::new()
        .await
        .map_err(|e| LincyError::Portal(format!("proxy: {}", e)))?;

    let session = proxy
        .create_session(CreateSessionOptions::default())
        .await
        .map_err(|e| LincyError::Portal(format!("session: {}", e)))?;

    let shortcuts = vec![NewShortcut::new("lincy-toggle", "Toggle Lincy clipboard manager")
        .preferred_trigger(shortcut)];

    proxy
        .bind_shortcuts(
            &session,
            &shortcuts,
            None::<&ashpd::WindowIdentifier>,
            BindShortcutsOptions::default(),
        )
        .await
        .map_err(|e| LincyError::Portal(e.to_string()))?;

    log::info!("Global shortcut '{}' registered via portal", shortcut);

    let mut stream = proxy
        .receive_activated()
        .await
        .map_err(|e| LincyError::Portal(format!("listen: {}", e)))?;

    while let Some(event) = stream.next().await {
        if event.shortcut_id() == "lincy-toggle" {
            log::debug!("Shortcut activated");
            let _ = tx.send(());
        }
    }

    Ok(())
}
