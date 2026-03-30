use std::path::{Path, PathBuf};
use chrono;
use std::fs;
use std::io::{Read, Write};
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
    pub file_size: u64,
    pub dest_path: String,
    pub file_progress: f64,
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
    pub xml_copied: usize,
    pub destination: String,
    pub brands: std::collections::HashMap<String, usize>,
    pub metadata_sources: std::collections::HashMap<String, usize>,
}

pub struct IngestState {
    pub cancel_flag: Arc<AtomicBool>,
}

#[tauri::command]
pub async fn cancel_ingest(state: tauri::State<'_, IngestState>) -> Result<(), String> {
    state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

fn copy_with_progress<F>(
    source: &Path, 
    dest: &Path, 
    mut on_progress: F
) -> std::io::Result<u64> 
where F: FnMut(u64, u64) {
    let mut src = fs::File::open(source)?;
    let total_size = src.metadata()?.len();
    let mut dst = fs::File::create(dest)?;
    let mut buffer = [0; 64 * 1024]; // 64KB buffer
    let mut copied = 0u64;

    while copied < total_size {
        let bytes_read = src.read(&mut buffer)?;
        if bytes_read == 0 { break; }
        dst.write_all(&buffer[..bytes_read])?;
        copied += bytes_read as u64;
        on_progress(copied, total_size);
    }
    
    // Ensure final 100% progress hit
    on_progress(total_size, total_size);
    Ok(total_size)
}

#[tauri::command]
pub async fn start_manual_ingest(app: AppHandle, drive_path: String, config: Config, state: tauri::State<'_, IngestState>) -> Result<(), String> {
    let dest_base = PathBuf::from(&config.dest_path);
    if !dest_base.exists() {
        return Err("Destination path does not exist. Please set it in Settings.".into());
    }

    state.cancel_flag.store(false, Ordering::SeqCst);
    
    // Recursively scan all folders to detect all photo and video structures
    let mut ingest_queue: Vec<(PathBuf, Vec<PathBuf>)> = Vec::new();
    let all_entries: Vec<walkdir::DirEntry> = WalkDir::new(&drive_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    // 1. Find all media files
    for entry in &all_entries {
        if config.is_allowed(entry.path()) {
            let mut sidecars = Vec::new();
            let parent = entry.path().parent().unwrap();
            let stem = entry.path().file_stem().and_then(|s| s.to_str()).unwrap_or("");
            
            // 2. Look for XML sidecars for this specific media file
            if !stem.is_empty() {
                for possible in &all_entries {
                    let p_path = possible.path();
                    if p_path.parent() == Some(parent) {
                        let p_ext = p_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                        if p_ext == "xml" {
                            let p_name = p_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            // Match {Stem}.XML or {Stem}M01.XML
                            if p_name == format!("{}.XML", stem) || p_name == format!("{}.xml", stem) || 
                                p_name == format!("{}M01.XML", stem) || p_name == format!("{}m01.xml", stem) {
                                sidecars.push(p_path.to_path_buf());
                            }
                        }
                    }
                }
            }
            ingest_queue.push((entry.path().to_path_buf(), sidecars));
        }
    }

    let total = ingest_queue.len();
    if total == 0 { return Ok(()); }

    let mut total_bytes = 0u64;
    for (source, _) in &ingest_queue {
        total_bytes += fs::metadata(source).map(|m| m.len()).unwrap_or(0);
    }

    let cancel_flag = state.cancel_flag.clone();
    tokio::task::spawn_blocking(move || {
        let mut target_dir_to_sync: Option<PathBuf> = None;
        let mut copied = 0usize;
        let mut skipped = 0usize;
        let mut photos_copied = 0usize;
        let mut raw_copied = 0usize;
        let mut videos_copied = 0usize;
        let mut xml_copied = 0usize;
        let mut brands: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut metadata_sources: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut bytes_copied: u64 = 0;
        let start_time = Instant::now();
        let mut dir_brand_cache: std::collections::HashMap<PathBuf, (String, String)> = std::collections::HashMap::new();
        let mut session_path_cache: std::collections::HashMap<PathBuf, PathBuf> = std::collections::HashMap::new();
        let session_import_date = chrono::Local::now().format("%Y-%m-%d").to_string();

        for (i, (source, sidecars)) in ingest_queue.iter().enumerate() {
            if cancel_flag.load(Ordering::Relaxed) {
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

                    if let Some(m) = &meta {
                        *metadata_sources.entry(m.source.clone()).or_insert(0) += 1;
                    }
                    
                    if parsed_brand != "Unknown" && !parsed_date.is_empty() {
                        dir_brand_cache.insert(parent_dir.clone(), (parsed_date.clone(), parsed_brand.clone()));
                    }
                    (parsed_date, parsed_brand)
                }
            } else {
                ("".to_string(), "Unknown".to_string())
            };

            let mut camera_dir = dest_base.clone();
            if config.organize_date && !date_str.is_empty() {
                camera_dir.push(&date_str);
            }
            if config.organize_brand {
                camera_dir.push(&brand);
            }

            // Suffix the camera/brand folder for the entire session
            let camera_dir_suffixed = if let Some(cached) = session_path_cache.get(&camera_dir) {
                cached.clone()
            } else {
                let resolved = get_suffixed_path(&camera_dir);
                session_path_cache.insert(camera_dir.clone(), resolved.clone());
                resolved
            };

            let mut final_target_dir = camera_dir_suffixed.clone();
            if target_dir_to_sync.is_none() {
                target_dir_to_sync = Some(camera_dir_suffixed);
            }

            // Determine file type
            let ext = source.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            let is_video = matches!(ext.as_str(), "mp4"|"mov"|"mts"|"mxf");

            if config.organize_brand {
                // Video separation logic (if organize_brand is ON, put Video inside brand folder)
                if is_video && config.video_folder == "separate" {
                    final_target_dir.push("Video");
                }

                // JPEG/RAW separation logic
                if config.separate_jpeg_raw && !is_video {
                    let is_raw = matches!(ext.as_str(), "arw"|"cr2"|"cr3"|"nef"|"raf"|"dng"|"orf"|"rw2"|"pef"|"srw"|"nrw"|"rwl");
                    let is_jpg = matches!(ext.as_str(), "jpg"|"jpeg");
                    if is_raw {
                        final_target_dir.push("RAW");
                    } else if is_jpg {
                        final_target_dir.push("JPEG");
                    }
                }
            } else {
                // If organize_brand is OFF, put Video in a top-level Video folder
                if is_video && config.video_folder == "separate" {
                    final_target_dir.push("Video");
                }
            }

            let _ = fs::create_dir_all(&final_target_dir);

            if let Some(filename) = source.file_name() {
                let dest_path = final_target_dir.join(filename);

                // Duplicate check
                let src_size  = fs::metadata(source).map(|m| m.len()).unwrap_or(0);
                let dest_size = fs::metadata(&dest_path).map(|m| m.len()).unwrap_or(u64::MAX);
                let is_dup    = src_size > 0 && src_size == dest_size;

                if is_dup {
                    skipped += 1;
                    // Emit progress for skipped file to keep UI moving
                    let current_bytes_copied = bytes_copied;
                    let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                    let speed = (current_bytes_copied as f64 / 1_048_576.0) / elapsed;
                    
                    let remaining_bytes = total_bytes.saturating_sub(current_bytes_copied);
                    let bytes_per_sec = (current_bytes_copied as f64) / elapsed;
                    let eta = if bytes_per_sec > 1024.0 { remaining_bytes as f64 / bytes_per_sec } else { 0.0 };

                    let _ = app.emit("copy-progress", ProgressPayload {
                        current: i + 1,
                        total,
                        current_file: filename.to_string_lossy().to_string(),
                        file_size: src_size,
                        dest_path: dest_path.to_string_lossy().to_string(),
                        file_progress: 1.0,
                        speed_mbps: speed,
                        eta_seconds: eta,
                    });
                } else {
                    let mut last_emit = Instant::now();
                    let app_h = app.clone();
                    let fname = filename.to_string_lossy().to_string();
                    let current_idx = i + 1;
                    let total_files = total;
                    let start_t = start_time.clone();
                    let bytes_before = bytes_copied;
                    let current_file_size = src_size;
                    let current_dest = dest_path.to_string_lossy().to_string();

                    let copy_res = copy_with_progress(source, &dest_path, |f_copied, f_total| {
                        let now = Instant::now();
                        if now.duration_since(last_emit).as_millis() > 100 || f_copied == f_total {
                            last_emit = now;
                            let f_progress = if f_total > 0 { f_copied as f64 / f_total as f64 } else { 1.0 };
                            let current_bytes_copied = bytes_before + f_copied;
                            let elapsed = start_t.elapsed().as_secs_f64().max(0.001);
                            let speed = (current_bytes_copied as f64 / 1_048_576.0) / elapsed;
                            
                            let remaining_bytes = total_bytes.saturating_sub(current_bytes_copied);
                            let bytes_per_sec = (current_bytes_copied as f64) / elapsed;
                            let eta = if bytes_per_sec > 1024.0 { remaining_bytes as f64 / bytes_per_sec } else { 0.0 };

                            let _ = app_h.emit("copy-progress", ProgressPayload {
                                current: current_idx,
                                total: total_files,
                                current_file: fname.clone(),
                                file_size: current_file_size,
                                dest_path: current_dest.clone(),
                                file_progress: f_progress,
                                speed_mbps: speed,
                                eta_seconds: eta.max(0.0),
                            });
                        }
                    });

                    if copy_res.is_ok() {
                        copied += 1;
                        bytes_copied += src_size;
                        *brands.entry(brand.clone()).or_insert(0) += 1;

                        if is_video {
                            videos_copied += 1;
                        } else if matches!(ext.as_str(), "arw"|"cr2"|"cr3"|"nef"|"raf"|"dng"|"orf"|"rw2"|"pef"|"srw"|"nrw"|"rwl") {
                            raw_copied += 1;
                        } else {
                            photos_copied += 1;
                        }

                        // Handle sidecars (XML) - Copy alongside and rename consistently
                        for sidecar in sidecars {
                            if let Some(sc_name) = sidecar.file_name() {
                                let sc_dest = final_target_dir.join(sc_name);
                                if fs::copy(sidecar, sc_dest).is_ok() {
                                    xml_copied += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        let _ = app.emit("copy-complete", CompletePayload {
            copied,
            skipped,
            photos_copied,
            raw_copied,
            videos_copied,
            xml_copied,
            destination: config.dest_path.clone(),
            brands,
            metadata_sources,
        });

        // Kick off remote sync if enabled
        if config.sync_remote && !config.remote_path.is_empty() {
            let sync_source = target_dir_to_sync.unwrap_or(dest_base);
            crate::sync::start_sync(app, config.remote_path.clone(), sync_source);
        }

        // Webhook notification
        if config.webhook_enabled && !config.webhook_url.is_empty() {
            let (title, folder_lbl, copied_lbl, skipped_lbl, xml_lbl) = if config.language == "vi" {
                ("📸 Nhập ảnh hoàn tất!", "Thư mục đích", "Đã chép", "Đã bỏ qua", "XML sidecar")
            } else {
                ("📸 Photo Ingest Complete!", "Destination", "Copied", "Skipped", "XML sidecars")
            };
            let msg = format!(
                "📂 **{}**: `{}`\n✅ **{}**: `{} tệp`\n⏭️ **{}**: `{} tệp`\n📄 **{}**: `{} tệp`",
                folder_lbl, config.dest_path, copied_lbl, copied, skipped_lbl, skipped, xml_lbl, xml_copied
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
