use tauri::{AppHandle, Emitter};
use std::thread;

#[derive(Clone, serde::Serialize)]
pub struct SDCardPayload {
    pub drive_path: String,
    pub volume_label: String,
    pub file_count: usize,
}

pub fn start_watcher(app: AppHandle) {
    #[cfg(target_os = "windows")]
    {
        thread::spawn(move || {
            windows_watcher(app);
        });
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        eprintln!("[FrameDrop] SD card watching is Windows-only in this build.");
    }
}

#[cfg(target_os = "windows")]
fn windows_watcher(app: AppHandle) {
    use wmi::{COMLibrary, WMIConnection};
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct Win32LogicalDisk {
        name: String,
        volume_name: Option<String>,
    }

    let com_con = match COMLibrary::new() {
        Ok(c) => c,
        Err(e) => { eprintln!("[FrameDrop] COM init failed: {e}"); return; }
    };
    let wmi_con = match WMIConnection::new(com_con) {
        Ok(w) => w,
        Err(e) => { eprintln!("[FrameDrop] WMI connection failed: {e}"); return; }
    };

    let mut seen_drives: std::collections::HashSet<String> = std::collections::HashSet::new();

    loop {
        // Query removable drives (DriveType = 2)
        let query: Result<Vec<Win32LogicalDisk>, _> = wmi_con.raw_query(
            "SELECT Name, VolumeName FROM Win32_LogicalDisk WHERE DriveType = 2"
        );

        if let Ok(disks) = query {
            for disk in &disks {
                let path = format!("{}\\", disk.name);
                if seen_drives.contains(&path) { continue; }

                let dcim = std::path::Path::new(&path).join("DCIM");
                if dcim.exists() && dcim.is_dir() {
                    let file_count = walkdir::WalkDir::new(&dcim)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                        .count();

                    if file_count > 0 {
                        seen_drives.insert(path.clone());
                        let _ = app.emit("sd-card-detected", SDCardPayload {
                            drive_path:   path,
                            volume_label: disk.volume_name.clone().unwrap_or_else(|| "SD Card".to_string()),
                            file_count,
                        });
                    }
                }
            }

            // Remove drives from seen set when they are ejected
            seen_drives.retain(|p| std::path::Path::new(p).exists());
        }

        thread::sleep(std::time::Duration::from_secs(3));
    }
}
