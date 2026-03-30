use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use chrono;

#[derive(Debug)]
pub struct PhotoMetadata {
    pub model: String,
    pub date_original: String,
}

pub fn read_metadata(path: &Path) -> Option<PhotoMetadata> {
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
