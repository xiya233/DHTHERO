# Nginx Host Reverse Proxy Guide (Subdomain + Certbot)

This guide uses **host-installed Nginx** (not Dockerized Nginx) to expose:

- `https://app.example.com` -> frontend (`127.0.0.1:3000`)
- `https://api.example.com` -> backend (`127.0.0.1:8080`)

It assumes your app stack is running with Docker Compose and that Nginx is installed directly on the host.

## 1) Prerequisites

- Domain DNS is ready:
  - `app.example.com` A record -> your server public IP
  - `api.example.com` A record -> your server public IP
- Docker stack is already running:

```bash
docker compose up -d
```

- Cloud/security group allows inbound `80` and `443`
- Host has sudo/root access

## 2) Install Nginx + Certbot (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install -y nginx certbot python3-certbot-nginx
```

Start and enable Nginx:

```bash
sudo systemctl enable --now nginx
```

For RHEL/Alma/Rocky, install equivalent packages via `dnf`.

## 3) Configure Nginx reverse proxy

Create one config file:

```bash
sudo tee /etc/nginx/conf.d/dht.conf >/dev/null <<'NGINX'
map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}

# app.example.com (Frontend)
server {
    listen 80;
    listen [::]:80;
    server_name app.example.com;

    # Allow ACME challenge on HTTP for certbot
    location ^~ /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    # Temporary HTTP access before TLS is issued
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
    }
}

# api.example.com (Backend)
server {
    listen 80;
    listen [::]:80;
    server_name api.example.com;

    # Allow ACME challenge on HTTP for certbot
    location ^~ /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
    }
}
NGINX
```

Create ACME webroot and test config:

```bash
sudo mkdir -p /var/www/certbot
sudo nginx -t
sudo systemctl reload nginx
```

## 4) Request TLS certificates (Let's Encrypt)

Issue certificates for both subdomains and let certbot update Nginx automatically:

```bash
sudo certbot --nginx \
  -d app.example.com \
  -d api.example.com
```

After success, certbot will add HTTPS server blocks and HTTP->HTTPS redirect.

Verify Nginx and reload:

```bash
sudo nginx -t
sudo systemctl reload nginx
```

## 5) Enable and verify auto-renew

Dry-run renewal test:

```bash
sudo certbot renew --dry-run
```

Check timer:

```bash
systemctl list-timers | grep certbot
```

## 6) Link project config to API domain

Edit `docker/env/frontend.env`:

```env
NEXT_PUBLIC_API_BASE_URL=https://api.example.com
```

Restart frontend so env changes apply:

```bash
docker compose up -d frontend
```

## 7) Port hardening (recommended baseline)

Goal: public internet only reaches `80/443`, not `3000/8080`.

### Option A: UFW (Ubuntu/Debian common)

```bash
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw deny 3000/tcp
sudo ufw deny 8080/tcp
sudo ufw enable
sudo ufw status verbose
```

### Option B: cloud firewall / security group

Allow only:

- `22/tcp` (or your SSH port)
- `80/tcp`
- `443/tcp`

Remove public rules for `3000/tcp` and `8080/tcp`.

## 8) Validation checklist

### Nginx config

```bash
sudo nginx -t && sudo systemctl reload nginx
```

### HTTP redirect to HTTPS

```bash
curl -I http://app.example.com
curl -I http://api.example.com
```

Expect `301`/`308` to `https://...`.

### HTTPS reachability

```bash
curl -I https://app.example.com
curl -s https://api.example.com/api/v1/healthz
```

Expected backend response includes `{"status":"ok"...}`.

### Functional checks

- Open `https://app.example.com`
- Verify search/latest/trending/admin pages work
- Verify private mode/admin login still works

### Security checks

From external network, `3000` and `8080` should be unreachable.

## Notes

- Keep backend and frontend containers listening on `0.0.0.0` inside Docker; security is enforced by host firewall and Nginx exposure policy.
- If you enabled `PRIVATE_MODE_ENABLED=true`, ensure frontend and backend private passwords remain identical in their env files.
