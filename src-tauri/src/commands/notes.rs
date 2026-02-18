use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct NoteMeta {
    pub title: String,
    pub participants: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    pub id: String,
    pub content: String,
    pub meta: NoteMeta,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
    pub preview: String,
}

fn notes_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join("Brief")
}

fn ensure_notes_dir() -> Result<PathBuf, String> {
    let dir = notes_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

#[tauri::command]
pub fn list_notes() -> Result<Vec<NoteSummary>, String> {
    let dir = ensure_notes_dir()?;
    let mut notes = Vec::new();

    let entries = fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let id = path.file_stem().unwrap().to_string_lossy().to_string();
            let meta_path = dir.join(format!("{}.meta.json", id));

            let content = fs::read_to_string(&path).unwrap_or_default();
            let preview: String = content.chars().take(120).collect();

            let meta: NoteMeta = if meta_path.exists() {
                let raw = fs::read_to_string(&meta_path).unwrap_or_default();
                serde_json::from_str(&raw).unwrap_or_else(|_| default_meta(&id))
            } else {
                default_meta(&id)
            };

            notes.push(NoteSummary {
                id,
                title: meta.title,
                created_at: meta.created_at,
                updated_at: meta.updated_at,
                tags: meta.tags,
                preview,
            });
        }
    }

    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(notes)
}

#[tauri::command]
pub fn read_note(id: String) -> Result<Note, String> {
    let dir = ensure_notes_dir()?;
    let md_path = dir.join(format!("{}.md", id));
    let meta_path = dir.join(format!("{}.meta.json", id));

    let content = fs::read_to_string(&md_path).map_err(|e| e.to_string())?;
    let meta: NoteMeta = if meta_path.exists() {
        let raw = fs::read_to_string(&meta_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&raw).map_err(|e| e.to_string())?
    } else {
        default_meta(&id)
    };

    Ok(Note { id, content, meta })
}

#[tauri::command]
pub fn write_note(id: String, content: String, meta: NoteMeta) -> Result<(), String> {
    let dir = ensure_notes_dir()?;
    let md_path = dir.join(format!("{}.md", id));
    let meta_path = dir.join(format!("{}.meta.json", id));

    fs::write(&md_path, &content).map_err(|e| e.to_string())?;
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(&meta_path, meta_json).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_note(id: String) -> Result<(), String> {
    let dir = notes_dir();
    let md_path = dir.join(format!("{}.md", id));
    let meta_path = dir.join(format!("{}.meta.json", id));

    if md_path.exists() {
        fs::remove_file(&md_path).map_err(|e| e.to_string())?;
    }
    if meta_path.exists() {
        fs::remove_file(&meta_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn default_meta(id: &str) -> NoteMeta {
    NoteMeta {
        title: id.to_string(),
        participants: vec![],
        tags: vec![],
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    }
}
