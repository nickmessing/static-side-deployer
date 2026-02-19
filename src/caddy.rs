use std::path::Path;

pub async fn upsert_config(caddy_dir: &Path, full_domain: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let config_dir = caddy_dir.join("config/static");
    tokio::fs::create_dir_all(&config_dir).await?;

    let config_path = config_dir.join(format!("{full_domain}.Caddyfile"));
    let new_content = generate_config(caddy_dir, full_domain);

    let changed = match tokio::fs::read_to_string(&config_path).await {
        Ok(existing) => existing != new_content,
        Err(_) => true,
    };

    if changed {
        tokio::fs::write(&config_path, &new_content).await?;
        restorecon(&config_path).await;
        tracing::info!("wrote caddy config: {}", config_path.display());
    }

    Ok(changed)
}

fn generate_config(caddy_dir: &Path, full_domain: &str) -> String {
    let caddy_dir = caddy_dir.display();
    format!(
        "{full_domain} {{\n\
        \x20   root * {caddy_dir}/static/{full_domain}\n\
        \x20   try_files {{path}} /index.html\n\
        \x20   file_server\n\
        }}\n"
    )
}

async fn restorecon(path: &Path) {
    match tokio::process::Command::new("restorecon")
        .arg("-R")
        .arg(path)
        .status()
        .await
    {
        Ok(status) if status.success() => {}
        Ok(status) => tracing::warn!("restorecon exited with {status} for {}", path.display()),
        Err(e) => tracing::warn!("restorecon failed for {}: {e}", path.display()),
    }
}

pub async fn reload_caddy() -> Result<(), Box<dyn std::error::Error>> {
    let conn = zbus::Connection::system().await?;
    let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn).await?;
    manager
        .reload_or_restart_unit("caddy.service".to_string(), "replace".to_string())
        .await?;
    tracing::info!("caddy reload triggered");
    Ok(())
}
