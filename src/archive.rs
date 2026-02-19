use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use crate::manifest::Manifest;

pub struct ArchiveContents {
    pub manifest: Manifest,
    pub files: HashMap<PathBuf, Vec<u8>>,
}

pub fn extract_archive(data: &[u8]) -> Option<ArchiveContents> {
    let decoder = flate2::read::GzDecoder::new(Cursor::new(data));
    let mut archive = tar::Archive::new(decoder);
    let entries = archive.entries().ok()?;

    let mut raw_files = HashMap::new();
    let mut root_items = HashSet::new();

    for entry in entries {
        let mut entry = entry.ok()?;
        let path = entry.path().ok()?.to_path_buf();

        let top = path
            .components()
            .next()?
            .as_os_str()
            .to_string_lossy()
            .to_string();
        root_items.insert(top);

        if entry.header().entry_type().is_file() {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).ok()?;
            raw_files.insert(path, buf);
        }
    }

    if root_items.len() != 2 || !root_items.contains("manifest.json") || !root_items.contains("dist")
    {
        return None;
    }

    let manifest_data = raw_files.remove(Path::new("manifest.json"))?;
    let manifest: Manifest = serde_json::from_slice(&manifest_data).ok()?;

    if manifest.domain.is_empty() {
        return None;
    }

    let dist_prefix = Path::new("dist");
    let files = raw_files
        .into_iter()
        .filter_map(|(path, data)| {
            let stripped = path.strip_prefix(dist_prefix).ok()?;
            Some((stripped.to_path_buf(), data))
        })
        .collect();

    Some(ArchiveContents { manifest, files })
}
