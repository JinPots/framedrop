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
fn test_remote_connection(_host: String, path: String) -> Result<String, String> {
    let remote_path = std::path::PathBuf::from(&path);
    if remote_path.exists() {
        Ok("Successfully reached network share.".to_string())
    } else {
        Err(format!("Path not reachable: {}", path))
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
                .build(app)?;

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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
