# VLT — Developer Secrets Vault

A zero-knowledge secrets vault for developers. Manage passwords, API keys, env vars, and SSH keys with client-side encryption.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Clients                                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │ Web      │  │ Desktop  │  │ Mobile (future)  │  │
│  │ SvelteKit│  │ Tauri    │  │                  │  │
│  └────┬─────┘  └────┬─────┘  └──────────────────┘  │
│       │              │                              │
│       │  Client-side AES-256-GCM encryption         │
│       │  (secrets never leave device unencrypted)   │
│       └──────┬───────┘                              │
└──────────────┼──────────────────────────────────────┘
               │ HTTPS
┌──────────────┼──────────────────────────────────────┐
│  PocketBase  │  (stores only ciphertext)            │
│  REST API + Auth + SQLite                           │
└─────────────────────────────────────────────────────┘
```

## Monorepo Structure

```
vlt/
├── frontend/          SvelteKit 5 + Tailwind CSS 4 + TypeScript
│   ├── src/
│   │   ├── lib/
│   │   │   ├── pb.ts              PocketBase client
│   │   │   ├── crypto.ts          AES-256-GCM + PBKDF2 key derivation
│   │   │   ├── auth.svelte.ts     Reactive auth store (Svelte 5 runes)
│   │   │   ├── types.ts           TypeScript types
│   │   │   └── components/
│   │   │       ├── SecretCard.svelte      Reveal, copy, delete
│   │   │       ├── AddSecretModal.svelte  Create new secrets
│   │   │       └── ImportModal.svelte     CSV import from browsers
│   │   └── routes/
│   │       ├── +page.svelte       Vault dashboard
│   │       ├── login/             Email + master password
│   │       └── signup/            Account creation
│   └── vercel.json
├── app/               Tauri 2 desktop shell
│   ├── src-tauri/
│   │   ├── src/                   Rust entry point
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json
│   └── package.json
├── db/                PocketBase
│   ├── pocketbase                 Binary (not committed)
│   └── pb_migrations/             Schema migrations
├── Cargo.toml         Rust workspace
├── package.json       pnpm workspace scripts
└── pnpm-workspace.yaml
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Web Frontend | SvelteKit 5, Svelte 5 Runes, TypeScript, Tailwind CSS 4, Vite 7 |
| Desktop | Tauri 2 (Rust + WebView) |
| Database | PocketBase 0.36 (Go + SQLite) |
| Encryption | Web Crypto API — AES-256-GCM, PBKDF2 (600k iterations) |
| Package Manager | pnpm workspaces |
| Deployment | Vercel (web), Vultr VPS (PocketBase), native installers (desktop) |

## Security Model

- **Zero-knowledge:** PocketBase stores only ciphertext + IVs. The encryption key never leaves the client.
- **Key derivation:** Master password → PBKDF2 (600,000 iterations, SHA-256) → 256-bit AES key.
- **Encryption:** AES-256-GCM with unique 12-byte IV per secret.
- **Key persistence:** Derived key stored in `sessionStorage` (cleared on tab close or logout).
- **Salt:** Random 16-byte salt per user, stored in PocketBase user record.

## PocketBase Schema

### `users` (auth collection)
| Field | Type | Notes |
|-------|------|-------|
| email | email | Login identity |
| password | password | PocketBase managed |
| encryption_salt | text | Base64, used for PBKDF2 key derivation |

### `secrets` (base collection)
| Field | Type | Notes |
|-------|------|-------|
| name | text | Plaintext label |
| type | select | password, api_key, env_var, ssh_key, note |
| encrypted_value | text | Base64 AES-256-GCM ciphertext |
| iv | text | Base64 initialization vector |
| username | text | Optional login username |
| url | url | Optional associated URL |
| user | relation | → users |

## Getting Started

### Prerequisites

- Node.js 20+
- pnpm 9+
- Rust 1.77+ (for desktop app)

### Install

```bash
git clone https://github.com/darwin808/vlt.git
cd vlt
pnpm install
```

### Download PocketBase

```bash
# macOS ARM
cd db
curl -L -o pb.zip https://github.com/pocketbase/pocketbase/releases/download/v0.36.7/pocketbase_0.36.7_darwin_arm64.zip
unzip pb.zip && rm pb.zip CHANGELOG.md LICENSE.md
chmod +x pocketbase

# Linux x86_64
curl -L -o pb.zip https://github.com/pocketbase/pocketbase/releases/download/v0.36.7/pocketbase_0.36.7_linux_amd64.zip
unzip pb.zip && rm pb.zip CHANGELOG.md LICENSE.md
chmod +x pocketbase
```

### Run Locally

```bash
# Terminal 1: Start PocketBase
pnpm db

# Terminal 2: Start frontend dev server
pnpm dev
```

Open http://localhost:5173 — sign up, then start adding secrets.

### Desktop App

```bash
# Development (start PocketBase first)
pnpm db              # Terminal 1
pnpm dev             # Terminal 2
pnpm desktop         # Terminal 3

# Production build
pnpm desktop:build
```

The built app is at `app/src-tauri/target/release/bundle/`.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `⌘N` / `Ctrl+N` | Add new secret |
| `/` | Focus search |
| `Esc` | Close modal |

## Browser Import

Supports CSV export from:
- **Brave / Chrome / Edge** — Settings → Passwords → Export
- **Firefox** — Settings → Passwords → Export
- **Safari** — Settings → Passwords → Export All Passwords

Click **Import** in the vault header, upload the CSV, preview, then import. All passwords are encrypted client-side before saving.

## Deployment

### Frontend (Vercel)

Deployed at: https://frontend-nu-orcin-51.vercel.app

```bash
cd frontend
vercel --yes --prod
```

Set `VITE_PB_URL` environment variable in Vercel to point at your PocketBase instance.

### PocketBase (Vultr VPS)

Running at: `139.180.216.160:8090` (via Cloudflare Tunnel for HTTPS)

```bash
# On VPS: systemd service
sudo systemctl status pocketbase      # Check status
sudo systemctl restart pocketbase     # Restart

# Cloudflare Tunnel (HTTPS)
sudo systemctl status cloudflared-pb  # Check tunnel
sudo journalctl -u cloudflared-pb     # Get tunnel URL
```

### VPS Services

| Service | Port | Manager | Notes |
|---------|------|---------|-------|
| PocketBase | 8090 | systemd | VLT database |
| Cloudflare Tunnel | — | systemd | HTTPS proxy for PocketBase |
| strategy-101 | 3000 | Docker | Separate project, untouched |

## Environment Variables

| Variable | Where | Purpose |
|----------|-------|---------|
| `VITE_PB_URL` | Frontend (Vercel) | PocketBase API URL. Defaults to `http://127.0.0.1:8090` for local dev. |

## Scripts

```bash
pnpm dev            # Frontend dev server (localhost:5173)
pnpm build          # Build frontend
pnpm db             # Start PocketBase (localhost:8090)
pnpm desktop        # Launch Tauri desktop app
pnpm desktop:build  # Build desktop app binary
```

## Roadmap

- [ ] Global hotkey (Cmd+Shift+V) → search → copy
- [ ] CLI: `vlt get`, `vlt env inject`, `vlt ssh list`
- [ ] Upgrade PBKDF2 → Argon2id
- [ ] Mobile app
- [ ] Team sync ($8/user/mo)
