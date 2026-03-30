pub mod config;
pub mod exif;
pub mod ingest;
pub mod sync;
pub mod watcher;

use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
fn open_folder(app: tauri::AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn test_remote_connection(_app: tauri::AppHandle, path: String) -> Result<String, String> {
    let remote_path = std::path::PathBuf::from(&path);
    
    // 1. Basic Path Format Check
    if !path.starts_with("\\\\") && !path.starts_with("//") {
        return Err("Invalid SMB path format. Use \\\\host\\share".to_string());
    }

    // 2. Host Reachability Check
    let host = path.trim_start_matches('\\').trim_start_matches('/').split('\\').next().unwrap_or_default();
    if !host.is_empty() {
        use std::net::{TcpStream, ToSocketAddrs};
        let addr = format!("{}:445", host);
        if let Ok(mut addrs) = addr.to_socket_addrs() {
            if let Some(sock_addr) = addrs.next() {
                if TcpStream::connect_timeout(&sock_addr, std::time::Duration::from_secs(2)).is_err() {
                    return Err(format!("Host '{}' not reachable on port 445 (SMB).", host));
                }
            }
        }
    }

    // 3. Share Accessibility Check
    if !remote_path.exists() {
        return Err(format!("Share not reachable or access denied: {}", path));
    }

    // 4. Write Permission Check
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let test_dir = remote_path.join(format!("framedrop_test_{}", now));
    match std::fs::create_dir(&test_dir) {
        Ok(_) => {
            let _ = std::fs::remove_dir(&test_dir);
            Ok("Successfully reached network share with write permissions.".to_string())
        }
        Err(e) => Err(format!("Share is readable but not writable: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            watcher::start_watcher(app.handle().clone());

            let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let open_i = tauri::menu::MenuItem::with_id(app, "open", "Open FrameDrop", true, None::<&str>)?;
            let status_i = tauri::menu::MenuItem::with_id(app, "status", "Watch status: Active", false, None::<&str>)?;
            let pause_i = tauri::menu::MenuItem::with_id(app, "pause", "Pause / Resume watching", true, None::<&str>)?;
            let open_dest_i = tauri::menu::MenuItem::with_id(app, "open_dest", "Open destination folder", true, None::<&str>)?;
            
            let menu = tauri::menu::Menu::with_items(app, &[
                &open_i, 
                &tauri::menu::PredefinedMenuItem::separator(app)?,
                &status_i,
                &pause_i,
                &open_dest_i,
                &tauri::menu::PredefinedMenuItem::separator(app)?,
                &quit_i
            ])?;

            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "open_dest" => {
                        let config = crate::config::get_config(app.clone());
                        if !config.dest_path.is_empty() {
                            let _ = open_folder(app.clone(), config.dest_path);
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let config = crate::config::get_config(app.handle().clone());
            if config.launch_in_background {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            app.manage(ingest::IngestState {
                cancel_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::save_config,
            ingest::start_manual_ingest,
            ingest::cancel_ingest,
            ingest::test_webhook,
            open_folder,
            test_remote_connection,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let config = crate::config::get_config(window.app_handle().clone());
                if config.minimize_to_tray_on_close {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
