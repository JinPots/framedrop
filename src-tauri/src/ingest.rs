use std::path::{Path, PathBuf};
use chrono;
use std::fs;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::config::Config;
use crate::exif::read_metadata;

fn get_suffixed_path(base: &Path) -> PathBuf {
    if !base.exists() {
        return base.to_path_buf();
    }
    let mut i = 1;
    let base_str = base.to_string_lossy();
    loop {
        let candidate = PathBuf::from(format!("{} ({})", base_str, i));
        if !candidate.exists() {
            return candidate;
        }
        i += 1;
    }
}

fn send_webhook_notification(url: &str, ping_id: &str, title: &str, msg: &str) {
    let client = reqwest::blocking::Client::new();
    let body = if url.contains("discord.com/api/webhooks") {
        let content = if !ping_id.is_empty() { format!("<@{}>", ping_id) } else { "".to_string() };
        serde_json::json!({
            "content": content,
            "embeds": [{
                "title": title,
                "description": msg,
                "color": 1358310 // teal #14b8a6
            }]
        })
    } else if url.contains("api.telegram.org/bot") {
        let mention = if !ping_id.is_empty() { format!("<a href=\"tg://user?id={}\">@User</a>\n", ping_id) } else { "".to_string() };
        let full_msg = format!("{}<b>{}</b>\n\n{}", mention, title, msg);
        serde_json::json!({ "text": full_msg, "parse_mode": "HTML" })
    } else {
        serde_json::json!({ "text": format!("{}\n{}", title, msg) })
    };
    let _ = client.post(url).json(&body).send();
}

#[derive(Clone, serde::Serialize)]
pub struct ProgressPayload {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub speed_mbps: f64,
    pub eta_seconds: f64,
}

#[derive(Clone, serde::Serialize)]
pub struct CompletePayload {
    pub copied: usize,
    pub skipped: usize,
    pub photos_copied: usize,
    pub raw_copied: usize,
    pub videos_copied: usize,
    pub destination: String,
    pub brands: std::collections::HashMap<String, usize>,
}

pub struct IngestState {
    pub cancel_flag: Arc<AtomicBool>,
}

#[tauri::command]
pub async fn cancel_ingest(state: tauri::State<'_, IngestState>) -> Result<(), String> {
    state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn start_manual_ingest(app: AppHandle, drive_path: String, config: Config, state: tauri::State<'_, IngestState>) -> Result<(), String> {
    let dest_base = PathBuf::from(&config.dest_path);
    if !dest_base.exists() {
        return Err("Destination path does not exist. Please set it in Settings.".into());
    }

    state.cancel_flag.store(false, Ordering::SeqCst);
    let cancel = state.cancel_flag.clone();

    tokio::task::spawn_blocking(move || {
        let mut target_dir_to_sync: Option<PathBuf> = None;

        // Recursively scan all folders to detect all photo and video structures
        let mut files_to_copy: Vec<PathBuf> = Vec::new();
        for entry in WalkDir::new(&drive_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && config.is_allowed(entry.path()) {
                files_to_copy.push(entry.path().to_path_buf());
            }
        }

        let total = files_to_copy.len();
        let mut copied = 0usize;
        let mut skipped = 0usize;
        let mut photos_copied = 0usize;
        let mut raw_copied = 0usize;
        let mut videos_copied = 0usize;
        let mut brands: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut bytes_copied: u64 = 0;
        let start_time = Instant::now();
        let mut dir_brand_cache: std::collections::HashMap<PathBuf, (String, String)> = std::collections::HashMap::new();
        let mut session_path_cache: std::collections::HashMap<PathBuf, PathBuf> = std::collections::HashMap::new();
        let session_import_date = chrono::Local::now().format("%Y-%m-%d").to_string();

        for (i, source) in files_to_copy.iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }

            let parent_dir = source.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
            
            let (date_str, brand) = if config.organize_brand || config.organize_date {
                if let Some(cached) = dir_brand_cache.get(&parent_dir) {
                    cached.clone()
                } else {
                    let meta = read_metadata(source);
                    let parsed_brand = if config.organize_brand {
                        meta.as_ref().map(|m| m.model.clone()).unwrap_or_else(|| "Unknown".to_string())
                    } else {
                        "Unknown".to_string()
                    };

                    let parsed_date = if config.organize_date {
                        if config.date_source == "import" {
                            session_import_date.clone()
                        } else {
                            meta.as_ref().map(|m| m.date_original.clone()).unwrap_or_else(|| session_import_date.clone())
                        }
                    } else {
                        "".to_string()
                    };
                    
                    if parsed_brand != "Unknown" && !parsed_date.is_empty() {
                        dir_brand_cache.insert(parent_dir.clone(), (parsed_date.clone(), parsed_brand.clone()));
                    }
                    (parsed_date, parsed_brand)
                }
            } else {
                ("".to_string(), "Unknown".to_string())
            };

            let mut target_dir = dest_base.clone();
            if config.organize_date && !date_str.is_empty() {
                target_dir.push(&date_str);
                if target_dir_to_sync.is_none() {
                    target_dir_to_sync = Some(target_dir.clone());
                }
            }

            // Video separation logic
            let is_video = source.extension()
                .map(|e| {
                    let ext = e.to_string_lossy().to_lowercase();
                    matches!(ext.as_str(), "mp4"|"mov"|"mts"|"mxf")
                })
                .unwrap_or(false);

            if is_video && config.video_folder == "separate" {
                target_dir.push("Video");
            }

            if config.organize_brand {
                target_dir.push(&brand);
            }

            // Suffix logic: If this is the first time seeing this base target_dir in this session, resolve suffix
            let final_target_dir = if let Some(cached) = session_path_cache.get(&target_dir) {
                cached.clone()
            } else {
                let resolved = get_suffixed_path(&target_dir);
                session_path_cache.insert(target_dir.clone(), resolved.clone());
                resolved
            };

            let _ = fs::create_dir_all(&final_target_dir);

            if let Some(filename) = source.file_name() {
                let dest_path = final_target_dir.join(filename);

                // Duplicate check: same filename + same size
                let src_size  = fs::metadata(source).map(|m| m.len()).unwrap_or(0);
                let dest_size = fs::metadata(&dest_path).map(|m| m.len()).unwrap_or(u64::MAX);
                let is_dup    = src_size > 0 && src_size == dest_size;

                if is_dup {
                    skipped += 1;
                } else if fs::copy(source, &dest_path).is_ok() {
                    copied += 1;
                    bytes_copied += src_size;
                    *brands.entry(brand.clone()).or_insert(0) += 1;

                    let ext = source.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                    if matches!(ext.as_str(), "mp4"|"mov"|"mts"|"mxf") {
                        videos_copied += 1;
                    } else if matches!(ext.as_str(), "arw"|"cr2"|"cr3"|"nef"|"raf"|"dng"|"orf"|"rw2"|"pef"|"srw"|"nrw"|"rwl") {
                        raw_copied += 1;
                    } else {
                        photos_copied += 1;
                    }
                }

                // Progress emission
                let elapsed_secs = start_time.elapsed().as_secs_f64().max(0.001);
                let speed_mbps   = (bytes_copied as f64 / 1_048_576.0) / elapsed_secs;
                let files_done   = (i + 1) as f64;
                let files_sec    = files_done / elapsed_secs;
                let remaining    = (total - i - 1) as f64;
                let eta_seconds  = if files_sec > 0.0 { remaining / files_sec } else { 0.0 };

                let _ = app.emit("copy-progress", ProgressPayload {
                    current:      i + 1,
                    total,
                    current_file: filename.to_string_lossy().to_string(),
                    speed_mbps,
                    eta_seconds,
                });
            }
        }

        let _ = app.emit("copy-complete", CompletePayload {
            copied,
            skipped,
            photos_copied,
            raw_copied,
            videos_copied,
            destination: config.dest_path.clone(),
            brands,
        });

        // Kick off remote sync if enabled
        if config.sync_remote && !config.remote_path.is_empty() {
            let sync_source = target_dir_to_sync.unwrap_or(dest_base);
            crate::sync::start_sync(app, config.remote_path.clone(), sync_source);
        }

        // Webhook notification
        if config.webhook_enabled && !config.webhook_url.is_empty() {
            let (title, folder_lbl, copied_lbl, skipped_lbl) = if config.language == "vi" {
                ("📸 Nhập ảnh hoàn tất!", "Thư mục đích", "Đã chép", "Đã bỏ qua")
            } else {
                ("📸 Photo Ingest Complete!", "Destination", "Copied", "Skipped")
            };
            let msg = format!(
                "📂 **{}**: `{}`\n✅ **{}**: `{} tệp`\n⏭️ **{}**: `{} tệp`",
                folder_lbl, config.dest_path, copied_lbl, copied, skipped_lbl, skipped
            );
            send_webhook_notification(&config.webhook_url, &config.webhook_ping_id, title, &msg);
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn test_webhook(url: String, ping_id: String, language: String) -> Result<String, String> {
    if url.is_empty() { return Err("Webhook URL is empty".to_string()); }
    let (title, msg) = if language == "vi" {
        ("🔔 Kiểm tra Webhook FrameDrop", "FrameDrop đã sẵn sàng gửi tóm tắt nhập ảnh. Đây là một ví dụ về thông báo tóm tắt.")
    } else {
        ("🔔 FrameDrop Webhook Test", "FrameDrop is now ready to send ingestion summaries. This is an example of a rich embed session notification.")
    };
    tokio::task::spawn_blocking(move || {
        send_webhook_notification(&url, &ping_id, title, msg);
    }).await.map_err(|e| e.to_string())?;
    Ok("Test notification sent.".to_string())
}
