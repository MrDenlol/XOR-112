use crate::i18n::tr;
use crate::model::Circuit;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
const MAX_BYTES: u64 = 128 * 1024 * 1024;

pub fn load(path: &Path) -> Result<Circuit, String> {
    let file = File::open(path).map_err(|e| {
        if crate::i18n::is_english() {
            format!("Could not open file: {e}")
        } else {
            format!("Не удалось открыть файл: {e}")
        }
    })?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| {
            if crate::i18n::is_english() {
                format!("Read error: {e}")
            } else {
                format!("Ошибка чтения: {e}")
            }
        })?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(tr("Файл схемы слишком большой (предел 128 МБ).").into());
    }
    let c: Circuit = serde_json::from_slice(&bytes).map_err(|e| {
        if crate::i18n::is_english() {
            format!("Invalid circuit format: {e}")
        } else {
            format!("Неверный формат схемы: {e}")
        }
    })?;
    c.validate()?;
    Ok(c)
}

/// Write to a temporary file in the same directory; replace only after a full flush.
pub fn save(path: &Path, circuit: &Circuit) -> Result<(), String> {
    circuit.validate()?;
    let bytes = serde_json::to_vec_pretty(circuit).map_err(|e| e.to_string())?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent).map_err(|e| {
        if crate::i18n::is_english() {
            format!("Could not create file: {e}")
        } else {
            format!("Не удалось создать файл: {e}")
        }
    })?;
    tmp.write_all(&bytes)
        .and_then(|_| tmp.as_file().sync_all())
        .map_err(|e| {
            if crate::i18n::is_english() {
                format!("Write error: {e}")
            } else {
                format!("Ошибка записи: {e}")
            }
        })?;
    tmp.persist(path).map_err(|e| {
        if crate::i18n::is_english() {
            format!("Could not save circuit: {}", e.error)
        } else {
            format!("Не удалось сохранить схему: {}", e.error)
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expanded_alu_roundtrip_preserves_gate_edits() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Вентили.binary");
        let mut c = Circuit::alu64_example();
        let l = c.netlist.clone().unwrap();
        let id = l.result[0];
        c.wires.retain(|w| w.to != id);
        c.connect(l.a[0], id, 0).unwrap();
        let node = c.nodes.iter_mut().find(|n| n.id == id).unwrap();
        node.label = "Изменённый выход".into();
        node.x = 2340.;
        node.y = 560.;
        save(&path, &c).unwrap();
        let restored = load(&path).unwrap();
        assert_eq!(restored, c);
        assert_eq!(
            restored.evaluate().unwrap().values[&id],
            Some(crate::model::Bit::One)
        );
        eprintln!(
            "Expanded netlist: {} nodes, {} wires, {} bytes",
            c.nodes.len(),
            c.wires.len(),
            std::fs::metadata(&path).unwrap().len()
        );
    }
    #[test]
    fn alu64_roundtrip_and_legacy_compatibility() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("АЛУ.binary");
        let mut c = Circuit::legacy_alu64_example();
        let a = c.alu64.as_mut().unwrap();
        a.a = crate::alu64::Word64(u64::MAX);
        a.operation = crate::alu64::Operation::Sqrt;
        save(&path, &c).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded, c);
        assert_eq!(loaded.alu64.unwrap().evaluate().low, u32::MAX as u64);
        let legacy = Circuit::half_adder();
        save(&path, &legacy).unwrap();
        assert_eq!(load(&path).unwrap(), legacy);
        c.version = 1;
        assert!(c.validate().is_err());
        c.version = 2;
        c.nodes = legacy.nodes;
        assert!(c.validate().is_err());
    }
    #[test]
    fn unicode_roundtrip_and_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Схема.binary");
        let mut c = Circuit::half_adder();
        save(&path, &c).unwrap();
        assert_eq!(load(&path).unwrap(), c);
        c.nodes[0].value = c.nodes[0].value.flipped();
        save(&path, &c).unwrap();
        assert_eq!(load(&path).unwrap(), c);
        let text = std::fs::read_to_string(path).unwrap();
        assert!(text.contains("\"value\": \"1\"") || text.contains("\"value\": \"0\""));
    }
    #[test]
    fn reject_nonbinary_signals_and_invalid_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.binary");
        let json = serde_json::to_string(&Circuit::half_adder())
            .unwrap()
            .replace("\"value\":\"1\"", "\"value\":\"2\"");
        std::fs::write(&path, json).unwrap();
        assert!(load(&path).is_err());
        std::fs::write(&path, b"not a circuit").unwrap();
        assert!(load(&path).is_err());
        let mut c = Circuit::default();
        c.version = 99;
        std::fs::write(&path, serde_json::to_vec(&c).unwrap()).unwrap();
        assert!(load(&path).is_err());
    }
    #[test]
    fn invalid_save_preserves_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("safe.binary");
        let c = Circuit::half_adder();
        save(&path, &c).unwrap();
        let mut invalid = c.clone();
        invalid.nodes[0].x = f32::NAN;
        assert!(save(&path, &invalid).is_err());
        assert_eq!(load(&path).unwrap(), c);
    }
}
