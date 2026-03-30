use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub dest_path: String,
    pub organize_date: bool,
    pub organize_brand: bool,
    pub sync_remote: bool,
    pub remote_ip: String,
    pub remote_path: String,
    pub include_raw: bool,
    pub include_jpeg: bool,
    pub include_video: bool,
    pub notify_tray: bool,
    pub play_sound: bool,
    pub start_with_os: bool,
    pub launch_in_background: bool,
    pub minimize_to_tray: bool,
    pub auto_ingest: bool,
    pub remote_method: String,
    pub webhook_url: String,
    pub webhook_enabled: bool,
    pub webhook_ping_id: String,
    pub language: String,
    pub separate_video: bool,
    pub scan_video_dirs: bool,
    pub date_source: String,
    pub video_folder: String,
    pub separate_jpeg_raw: bool,
    pub minimize_to_tray_on_close: bool,
    pub remote_username: String,
    pub remote_password: String,
}

impl Config {
    pub fn is_allowed(&self, path: &Path) -> bool {
        if let Some(os_ext) = path.extension() {
            let e = os_ext.to_string_lossy().to_lowercase();
            let is_raw = matches!(e.as_str(), "arw"|"cr2"|"cr3"|"nef"|"raf"|"dng"|"orf"|"rw2"|"pef"|"srw"|"nrw"|"rwl");
            let is_jpg = matches!(e.as_str(), "jpg"|"jpeg");
            let is_vid = matches!(e.as_str(), "mp4"|"mov"|"mts"|"mxf");

            if is_raw && self.include_raw   { return true; }
            if is_jpg && self.include_jpeg  { return true; }
            if is_vid && self.include_video { return true; }
        }
        false
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dest_path: String::new(),
            organize_date: true,
            organize_brand: true,
            sync_remote: false,
            remote_ip: String::new(),
            remote_path: String::new(),
            include_raw: true,
            include_jpeg: true,
            include_video: true,
            notify_tray: true,
            play_sound: false,
            start_with_os: false,
            launch_in_background: false,
            minimize_to_tray: true,
            auto_ingest: false,
            remote_method: "SMB".to_string(),
            webhook_url: String::new(),
            webhook_enabled: false,
            webhook_ping_id: String::new(),
            language: "en".to_string(),
            separate_video: true,
            scan_video_dirs: true,
            date_source: "capture".to_string(),
            video_folder: "separate".to_string(),
            separate_jpeg_raw: true,
            minimize_to_tray_on_close: true,
            remote_username: String::new(),
            remote_password: String::new(),
        }
    }
}

pub fn get_config_path(app: &AppHandle) -> PathBuf {
    let mut path = app.path().app_data_dir().expect("Failed to get app data dir");
    fs::create_dir_all(&path).unwrap_or_default();
    path.push("config.json");
    path
}

#[tauri::command]
pub fn get_config(app: AppHandle) -> Config {
    let path = get_config_path(&app);
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str(&data) {
                return config;
            }
        }
    }
    Config::default()
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: Config) -> Result<(), String> {
    let path = get_config_path(&app);
    let data = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}
