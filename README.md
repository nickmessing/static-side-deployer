# static-site-deployer

Utility to deploy static sites to a VM behind Caddy.

Receives `.tgz` archives via HTTP POST, extracts them, configures Caddy, and serves the static files.

## Building

```bash
cargo build --release
```

The release profile is configured with LTO, single codegen unit, symbol stripping, and abort-on-panic for a smaller, faster binary. The output will be at `target/release/static-site-deployer`.

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
mkdir -p /etc/static-site-deployer
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

### 4. SELinux (Fedora/RHEL)

Set file contexts so Caddy can read the configs and static files we deploy:

```bash
semanage fcontext -a -t httpd_config_t "/etc/caddy/config(/.*)?"
semanage fcontext -a -t httpd_sys_content_t "/etc/caddy/static(/.*)?"
restorecon -Rv /etc/caddy/config /etc/caddy/static
```

The app automatically runs `restorecon -R` after writing files, so new deployments pick up the correct contexts. D-Bus access to systemd is permitted by default for services running as `unconfined_service_t`. If SELinux blocks D-Bus calls, check `ausearch -m avc -ts recent` and generate a policy with `audit2allow`.

### 5. Enable and start

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
