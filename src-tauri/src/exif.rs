use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use chrono;
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;

#[derive(Debug)]
pub struct PhotoMetadata {
    pub model: String,
    pub date_original: String,
    pub source: String,
}

pub fn read_xml_metadata(path: &Path) -> Option<PhotoMetadata> {
    if !path.exists() { return None; }
    let content = std::fs::read_to_string(path).ok()?;
    let doc = roxmltree::Document::parse(&content).ok()?;
    
    // Support Sony NonRealTimeMeta format
    let device_node = doc.descendants().find(|n| n.has_tag_name("Device"));
    if let Some(device) = device_node {
        let manufacturer = device.attribute("manufacturer").unwrap_or("");
        let model_name = device.attribute("modelName").unwrap_or("");
        
        let raw_model = if !model_name.is_empty() {
            model_name.to_string()
        } else if !manufacturer.is_empty() {
            normalize_brand(manufacturer)
        } else {
            "Unknown".to_string()
        };

        // Sanitize illegal path characters
        let model = raw_model.replace(&['/', '\\', ':', '*', '?', '"', '<', '>', '|'][..], "_");
        
        if model != "Unknown" {
            // Try to find creation date in XML
            let mut date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
            if let Some(creation) = doc.descendants().find(|n| n.has_tag_name("CreationDate")) {
                if let Some(val) = creation.attribute("value") {
                    if val.len() >= 10 {
                        date_str = val[..10].replace(':', "-").replace('/', "-");
                    }
                }
            } else if let Some(last_update) = doc.descendants().find(|n| n.has_tag_name("LastUpdate")) {
                if let Some(val) = last_update.attribute("value") {
                    if val.len() >= 10 {
                        date_str = val[..10].replace(':', "-").replace('/', "-");
                    }
                }
            }

            return Some(PhotoMetadata {
                model,
                date_original: date_str,
                source: "XML sidecar".to_string(),
            });
        }
    }
    
    None
}

pub fn read_metadata(path: &Path) -> Option<PhotoMetadata> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    let is_vid = matches!(ext.as_str(), "mp4"|"mov"|"mts"|"mxf");
    
    if is_vid {
        return read_video_metadata(path);
    }

    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let exifreader = exif::Reader::new();
    let exif_data = exifreader.read_from_container(&mut reader).ok()?;

    let make_tag  = exif_data.get_field(exif::Tag::Make,  exif::In::PRIMARY);
    let model_tag = exif_data.get_field(exif::Tag::Model, exif::In::PRIMARY);
    let date_tag  = exif_data.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY);

    let mut model_str = if let Some(m) = model_tag {
        format!("{}", m.display_value().with_unit(&exif_data)).trim_matches('"').trim().to_string()
    } else if let Some(mk) = make_tag {
        normalize_brand(&format!("{}", mk.display_value().with_unit(&exif_data)))
    } else {
        "Unknown".to_string()
    };
    
    // Sanitize illegal path characters from model name
    model_str = model_str.replace(&['/', '\\', ':', '*', '?', '"', '<', '>', '|'][..], "_");

    if model_str.is_empty() {
        model_str = "Unknown".to_string();
    }

    let date_str = if let Some(d) = date_tag {
        let val = format!("{}", d.display_value().with_unit(&exif_data));
        let val = val.trim_matches('"').trim();
        if val.len() >= 10 {
            let date_part = val.split(' ').next().unwrap_or("Unknown_Date");
            date_part.replace(':', "-")
        } else {
            chrono::Local::now().format("%Y-%m-%d").to_string()
        }
    } else {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    };

    Some(PhotoMetadata {
        model: model_str,
        date_original: date_str,
        source: "EXIF metadata".to_string(),
    })
}

pub fn read_video_metadata(path: &Path) -> Option<PhotoMetadata> {
    // 1. Try XML sidecar first
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if !stem.is_empty() {
        if let Some(parent) = path.parent() {
            // Check {Stem}.XML
            let xml_path = parent.join(format!("{}.XML", stem));
            if let Some(meta) = read_xml_metadata(&xml_path) {
                return Some(meta);
            }
            // Check {Stem}M01.XML (Sony)
            let sony_xml_path = parent.join(format!("{}M01.XML", stem));
            if let Some(meta) = read_xml_metadata(&sony_xml_path) {
                return Some(meta);
            }
        }
    }

    // 2. Fallback to container metadata
    let probe = Probe::open(path).ok()?;
    let tagged_file = probe.read().ok()?;
    
    let mut model_str = "Unknown".to_string();
    let mut date_str = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Prioritize tags for Sony/Canon/etc.
    if let Some(tag) = tagged_file.primary_tag() {
        // Brute force search for model/make in all tags
        for item in tag.items() {
            let key = format!("{:?}", item.key()).to_lowercase();
            if key.contains("model") || key.contains("make") || key.contains("device") {
                if let Some(val) = item.value().text() {
                    let trimmed = val.trim();
                    if !trimmed.is_empty() && trimmed != "Unknown" {
                        model_str = trimmed.to_string();
                        break;
                    }
                }
            }
        }

        // Try to get creation date
        for item in tag.items() {
            let key = format!("{:?}", item.key()).to_lowercase();
            if key.contains("recordingdate") || key.contains("encodingtime") || key.contains("created") {
                if let Some(d) = item.value().text() {
                    let d_trimmed = d.trim_matches(|c: char| c == '"').trim();
                    if d_trimmed.len() >= 10 {
                        date_str = d_trimmed[..10].replace(':', "-").replace('/', "-");
                        break;
                    }
                }
            }
        }
    }

    // Sanitize illegal path characters
    if model_str != "Unknown" {
        model_str = model_str.replace(&['/', '\\', ':', '*', '?', '"', '<', '>', '|'][..], "_");
    }
    
    // Fallback date to file modification
    if date_str == chrono::Local::now().format("%Y-%m-%d").to_string() {
        if let Ok(meta) = std::fs::metadata(path) {
            if let Ok(modified) = meta.modified() {
                let dt: chrono::DateTime<chrono::Local> = modified.into();
                date_str = dt.format("%Y-%m-%d").to_string();
            }
        }
    }

    Some(PhotoMetadata {
        model: model_str,
        date_original: date_str,
        source: "video metadata fallback".to_string(),
    })
}

fn normalize_brand(raw: &str) -> String {
    let p = raw.to_lowercase();
    let p = p.trim_matches('"').trim();
    if p.contains("sony") { "Sony".to_string() }
    else if p.contains("canon") { "Canon".to_string() }
    else if p.contains("nikon") { "Nikon".to_string() }
    else if p.contains("fuji") { "Fujifilm".to_string() }
    else if p.contains("panasonic") || p.contains("lumix") { "Panasonic".to_string() }
    else if p.contains("olympus") { "Olympus".to_string() }
    else if p.contains("leica") { "Leica".to_string() }
    else if p.contains("apple") { "Apple".to_string() }
    else if p.contains("hasselblad") { "Hasselblad".to_string() }
    else if p.contains("ricoh") || p.contains("pentax") { "Ricoh".to_string() }
    else if p.contains("sigma") { "Sigma".to_string() }
    else { "Unknown".to_string() }
}
