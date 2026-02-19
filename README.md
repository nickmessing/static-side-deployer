# static-site-deployer

Utility to deploy static sites to a VM behind Caddy.

Receives `.tgz` archives via HTTP POST, extracts them, configures Caddy, and serves the static files.

## Building

```bash
cargo build --release
```

The binary will be at `target/release/static-side-deployer`.

## Configuration

The app is configured via environment variables and systemd credentials:

| Variable | Required | Default | Description |
|---|---|---|---|
| `PORT` | No | `9092` | HTTP listen port |
| `CADDY_DIRECTORY` | Yes | — | Root directory for Caddy configs and static files |
| `CREDENTIALS_DIRECTORY` | Yes | — | Set automatically by systemd `LoadCredential` |

The secret token is read from `$CREDENTIALS_DIRECTORY/secret-token`.

## Systemd Setup

### 1. Create the token file

```bash
openssl rand -hex 32 > /etc/static-site-deployer/secret-token
chmod 600 /etc/static-site-deployer/secret-token
```

### 2. Create the systemd unit

Create `/etc/systemd/system/static-site-deployer.service`:

```ini
[Unit]
Description=Static Site Deployer
After=network.target caddy.service

[Service]
Type=simple
ExecStart=/usr/local/bin/static-site-deployer
Environment=PORT=9092
Environment=CADDY_DIRECTORY=/etc/caddy
LoadCredential=secret-token:/etc/static-site-deployer/secret-token
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

### 3. Configure Caddy

Add this to your main Caddyfile (e.g. `/etc/caddy/Caddyfile`) to import deployed site configs:

```
import /etc/caddy/config/static/*
```

### 4. Enable and start

```bash
systemctl daemon-reload
systemctl enable --now static-site-deployer
```

## Archive Format

The `/push` endpoint expects a `.tgz` archive with the following structure:

```
manifest.json
dist/
  index.html
  ...
```

### manifest.json

```json
{
  "scope": "public",
  "domain": "mysite"
}
```

| Field | Values | Description |
|---|---|---|
| `scope` | `"public"` or `"private"` | Public sites get `*.nickmessing.com`, private get `*.internal` |
| `domain` | string | Subdomain prefix |

A manifest with `scope: "public"` and `domain: "mysite"` deploys to `mysite.nickmessing.com`.

## API Usage

```bash
# Create a test archive
mkdir -p /tmp/site/dist
echo '{"scope":"public","domain":"mysite"}' > /tmp/site/manifest.json
echo '<h1>Hello</h1>' > /tmp/site/dist/index.html
tar -czf /tmp/site.tgz -C /tmp/site manifest.json dist

# Deploy
curl -X POST http://localhost:9092/push \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  --data-binary @/tmp/site.tgz
```
