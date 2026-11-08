use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "en")]
    English,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "gray")]
    Gray,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub language: Language,
    pub theme: Theme,
    pub grid: bool,
    pub snap: bool,
    pub frames: bool,
    pub animation: bool,
    pub zoom: f32,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            language: Language::Russian,
            theme: Theme::Green,
            grid: true,
            snap: true,
            frames: true,
            animation: true,
            zoom: 1.0,
        }
    }
}
pub fn path() -> Option<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
    };
    base.map(|p| p.join("XOR-112").join("settings.json"))
}
pub fn load_with_legacy(path: &Path) -> Result<Preferences, String> {
    if path.try_exists().map_err(|e| e.to_string())? {
        return load(path);
    }
    if let Some(base) = path.parent().and_then(Path::parent) {
        let legacy = base.join("BinaryLogic").join("settings.json");
        if legacy.try_exists().map_err(|e| e.to_string())? {
            let preferences = load(&legacy)?;
            save(path, &preferences)?;
            return Ok(preferences);
        }
    }
    load(path)
}
pub fn load(path: &Path) -> Result<Preferences, String> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Preferences::default()),
        Err(e) => return Err(e.to_string()),
    };
    let mut data = Vec::new();
    file.take(16385)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > 16384 {
        return Err("Settings file is too large".into());
    }
    let mut p: Preferences = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    if !p.zoom.is_finite() || !(0.005..=2.5).contains(&p.zoom) {
        p.zoom = 1.0;
    }
    Ok(p)
}
pub fn save(path: &Path, p: &Preferences) -> Result<(), String> {
    if !p.zoom.is_finite() || !(0.005..=2.5).contains(&p.zoom) {
        return Err("Invalid zoom".into());
    }
    let parent = path.parent().ok_or("Missing settings directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut tmp, p).map_err(|e| e.to_string())?;
    tmp.flush()
        .and_then(|_| tmp.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    tmp.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migrate_legacy_settings_once() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("BinaryLogic/settings.json");
        let new = dir.path().join("XOR-112/settings.json");
        let p = Preferences {
            language: Language::English,
            theme: Theme::Blue,
            zoom: 0.75,
            ..Default::default()
        };
        save(&old, &p).unwrap();
        assert_eq!(load_with_legacy(&new).unwrap(), p);
        assert_eq!(load(&new).unwrap(), p);
        assert!(old.exists());
        save(&old, &Preferences::default()).unwrap();
        assert_eq!(load_with_legacy(&new).unwrap(), p);
    }
    #[test]
    fn settings_roundtrip_and_invalid_input() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config/settings.json");
        assert_eq!(load(&path).unwrap(), Preferences::default());
        let p = Preferences {
            language: Language::English,
            theme: Theme::Blue,
            grid: false,
            snap: false,
            frames: false,
            animation: false,
            zoom: 0.75,
        };
        save(&path, &p).unwrap();
        assert_eq!(load(&path).unwrap(), p);
        std::fs::write(&path, r#"{"zoom":999,"theme":"gray"}"#).unwrap();
        let p = load(&path).unwrap();
        assert_eq!(p.zoom, 1.);
        assert_eq!(p.theme, Theme::Gray);
        std::fs::write(&path, "bad JSON").unwrap();
        assert!(load(&path).is_err());
    }
}
