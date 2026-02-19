use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub async fn deploy_files(
    caddy_dir: &Path,
    full_domain: &str,
    files: HashMap<PathBuf, Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = caddy_dir.join("static").join(full_domain);
    let staging = target.with_extension("new");
    let old = target.with_extension("old");

    // Clean up any leftover staging dir
    let _ = tokio::fs::remove_dir_all(&staging).await;

    // Write files to staging directory
    for (path, data) in &files {
        let dest = staging.join(path);
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&dest, data).await?;
    }

    // Swap: target -> old, staging -> target, remove old
    let target_exists = tokio::fs::metadata(&target).await.is_ok();
    if target_exists {
        tokio::fs::rename(&target, &old).await?;
    }
    tokio::fs::rename(&staging, &target).await?;
    if target_exists {
        let _ = tokio::fs::remove_dir_all(&old).await;
    }

    // Restore SELinux contexts so Caddy can read the files
    match tokio::process::Command::new("restorecon")
        .arg("-R")
        .arg(&target)
        .status()
        .await
    {
        Ok(status) if status.success() => {}
        Ok(status) => tracing::warn!("restorecon exited with {status} for {}", target.display()),
        Err(e) => tracing::warn!("restorecon failed for {}: {e}", target.display()),
    }

    tracing::info!(
        "deployed {} files to {}",
        files.len(),
        target.display()
    );

    Ok(())
}
