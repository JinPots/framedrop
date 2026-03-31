use std::path::{PathBuf};
use std::fs;
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

#[derive(Clone, serde::Serialize)]
pub struct SyncStartedPayload {
    pub total: usize,
    pub remote_path: String,
}

#[derive(Clone, serde::Serialize)]
pub struct SyncProgressPayload {
    pub current: usize,
    pub total: usize,
    pub percentage: f64,
    pub file_name: String,
}

#[derive(Clone, serde::Serialize)]
pub struct SyncCompletePayload {
    pub success: bool,
    pub error: Option<String>,
}

pub fn start_sync(app: AppHandle, remote_path: String, local_source: PathBuf) {
    tokio::task::spawn_blocking(move || {
        let remote_base = PathBuf::from(&remote_path);
        
        if !remote_base.exists() {
            let error_msg = if !remote_path.starts_with("\\\\") && !remote_path.starts_with("//") {
                format!("Invalid SMB path format: {}. Use \\\\host\\share", remote_path)
            } else {
                format!("Remote path {} unreachable. Ensure host is online and share is accessible.", remote_path)
            };

            let _ = app.emit("sync-complete", SyncCompletePayload { 
                success: false, 
                error: Some(error_msg) 
            });
            return;
        }
        
        // 1. Scan for total files first
        let total = WalkDir::new(&local_source)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count();

        let _ = app.emit("sync-started", SyncStartedPayload {
            total,
            remote_path: remote_path.clone(),
        });

        // 2. Start Syncing
        let dirname = local_source.file_name().unwrap_or_default();
        let target_dir = remote_base.join(dirname);
        let _ = fs::create_dir_all(&target_dir);
        
        let mut current = 0;
        for entry in WalkDir::new(&local_source).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let p = entry.path();
                if let Ok(rel) = p.strip_prefix(&local_source) {
                    let dest = target_dir.join(rel);
                    if let Some(parent) = dest.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    
                    let filename = p.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                    current += 1;
                    let percentage = (current as f64 / total.max(1) as f64) * 100.0;

                    let _ = app.emit("sync-progress", SyncProgressPayload {
                        current,
                        total,
                        percentage,
                        file_name: filename.to_string(),
                    });

                    // Duplicate check
                    let source_meta = fs::metadata(p).ok();
                    let target_meta = fs::metadata(&dest).ok();
                    let mut skip = false;
                    if let (Some(s_m), Some(t_m)) = (source_meta, target_meta) {
                        if s_m.len() == t_m.len() { skip = true; }
                    }
                    
                    if !skip {
                        let _ = fs::copy(p, &dest);
                    }
                }
            }
        }
        
        let _ = app.emit("sync-complete", SyncCompletePayload { 
            success: true, 
            error: None,
        });
    });
}
