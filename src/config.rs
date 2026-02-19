use std::path::PathBuf;

pub struct AppConfig {
    pub port: u16,
    pub caddy_directory: PathBuf,
    pub secret_token: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(9092);

        let caddy_directory = PathBuf::from(
            std::env::var("CADDY_DIRECTORY")
                .map_err(|_| "CADDY_DIRECTORY environment variable is required")?,
        );

        let credentials_dir = std::env::var("CREDENTIALS_DIRECTORY")
            .map_err(|_| "CREDENTIALS_DIRECTORY environment variable is required (systemd LoadCredential)")?;

        let token_path = PathBuf::from(&credentials_dir).join("secret-token");
        let secret_token = std::fs::read_to_string(&token_path)
            .map_err(|e| format!("failed to read secret token from {}: {e}", token_path.display()))?
            .trim()
            .to_string();

        if secret_token.is_empty() {
            return Err("secret token is empty".into());
        }

        Ok(Self {
            port,
            caddy_directory,
            secret_token,
        })
    }
}
