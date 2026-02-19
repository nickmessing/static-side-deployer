use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use crate::manifest::Manifest;

pub struct ArchiveContents {
    pub manifest: Manifest,
    pub files: HashMap<PathBuf, Vec<u8>>,
}

fn normalize_path(path: &Path) -> PathBuf {
    path.strip_prefix("./").unwrap_or(path).to_path_buf()
}

pub fn extract_archive(data: &[u8]) -> Result<ArchiveContents, String> {
    let decoder = flate2::read::GzDecoder::new(Cursor::new(data));
    let mut archive = tar::Archive::new(decoder);
    let entries = archive.entries().map_err(|e| format!("failed to read tar entries: {e}"))?;

    let mut manifest_data: Option<Vec<u8>> = None;
    let mut dist_files: HashMap<PathBuf, Vec<u8>> = HashMap::new();

    for (i, entry) in entries.enumerate() {
        let mut entry = entry.map_err(|e| format!("failed to read entry {i}: {e}"))?;
        let path = normalize_path(&entry.path().map_err(|e| format!("bad path in entry {i}: {e}"))?.to_path_buf());

        let is_file = entry.header().entry_type().is_file();

        if path == Path::new("manifest.json") && is_file {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("failed to read manifest.json: {e}"))?;
            manifest_data = Some(buf);
        } else if is_file {
            if let Ok(stripped) = path.strip_prefix("dist") {
                let mut buf = Vec::new();
                entry
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
                dist_files.insert(stripped.to_path_buf(), buf);
            } else {
                tracing::warn!("unexpected file outside dist/: {}", path.display());
            }
        }
    }

    let manifest_data = manifest_data.ok_or("manifest.json not found in archive")?;

    if dist_files.is_empty() {
        return Err("no files found in dist/ directory".into());
    }

    let manifest: Manifest =
        serde_json::from_slice(&manifest_data).map_err(|e| format!("invalid manifest.json: {e}"))?;

    if manifest.domain.is_empty() {
        return Err("manifest domain is empty".into());
    }

    tracing::info!(
        "extracted manifest (domain={}, scope={:?}) and {} dist files",
        manifest.domain,
        manifest.scope,
        dist_files.len()
    );

    Ok(ArchiveContents {
        manifest,
        files: dist_files,
    })
}
