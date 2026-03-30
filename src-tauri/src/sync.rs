use std::path::{PathBuf};
use std::fs;
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

#[derive(Clone, serde::Serialize)]
pub struct SyncCompletePayload {
    pub success: bool,
    pub error: Option<String>,
}

pub fn start_sync(app: AppHandle, remote_path: String, local_source: PathBuf) {
    tokio::task::spawn_blocking(move || {
        let remote_base = PathBuf::from(&remote_path);
        
        if !remote_base.exists() {
            let _ = app.emit("sync-complete", SyncCompletePayload { 
                success: false, 
                error: Some(format!("Remote path {} unreachable", remote_path)) 
            });
            return;
        }
        
        // Create matching folder structure manually if UNC path
        // For simplicity, we just copy everything from local_source inside it
        let dirname = local_source.file_name().unwrap_or_default();
        let target_dir = remote_base.join(dirname);
        let _ = fs::create_dir_all(&target_dir);
        
        for entry in WalkDir::new(&local_source).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let p = entry.path();
                if let Ok(rel) = p.strip_prefix(&local_source) {
                    let dest = target_dir.join(rel);
                    if let Some(parent) = dest.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    
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
