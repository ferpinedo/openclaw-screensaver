# OpenClaw Activity Screen

A real-time public activity display for OpenClaw, designed to showcase live events, messages, and system activity from your OpenClaw instance. This app connects directly to the OpenClaw gateway and visually presents activity updates on a dedicated screen—ideal for lobbies, offices, or any public display needing live operational visibility. No custom backend setup is required—just connect to your OpenClaw gateway, authenticate, and go!

## Quick start

### Recommended: OpenClaw on the same machine

For a lobby or office display, run OpenClaw and this app on **one computer**. You connect over loopback (`ws://127.0.0.1:18789`), use the gateway token from that install, and you do not need to expose the gateway on the network.

1. **Start OpenClaw** so the gateway WebSocket is listening (default: `ws://127.0.0.1:18789/`). Use the command or service your OpenClaw docs describe for running the gateway locally.

2. **Copy the gateway token** (same machine, in a terminal):

   ```bash
   openclaw config get gateway.auth.token
   ```

   Paste **only** the printed token into the app’s **Gateway token** field. That value is sent to the gateway as auth; do not paste shell commands into the field.

3. **Start this app** — from source:

   ```bash
   npm install
   npm run dev
   ```

   Or install a built `.deb`, AppImage, macOS `.app` bundle, or other platform bundle, then open **OpenClaw Activity Screen**.

4. Press **Ctrl+,** for settings. Optionally set **Screen title** and **Display name** (pairing / presence). Click **Use local OpenClaw** (fills `ws://127.0.0.1:18789`) or type that URL, paste the token, then **Save and connect**.

After a successful pair, the app stores a device token and reconnects on its own.

**Shortcuts:** **Ctrl+,** — settings; **Ctrl+.** — diagnostics panel.

### Advanced: remote or LAN gateway

Use this when the gateway runs on **another host** or you reach it over the LAN. In settings, set **Gateway URL** to that host’s `ws://…` or `wss://…`. Prefer **`wss://`** on untrusted networks. If your deployment uses it, a **setup code** can carry URL and bootstrap token instead of typing everything by hand. Auth and pairing work the same way as locally: use the token or bootstrap flow your server documents; the first connect may still need `openclaw devices list` and `openclaw devices approve <requestId>` on the gateway side.

```bash
openclaw qr --setup-code-only
openclaw devices list
openclaw devices approve <requestId>
```

## Product direction

The intended end-user flow is:

1. Install the app (or run from source).
2. Connect to OpenClaw (locally recommended; remote optional).
3. Leave the app running full-screen while OpenClaw pushes events in real time.

The public-machine lock down should happen at the OS level, not inside the app.

- Linux: dedicated kiosk or locked-down desktop account (see [Ubuntu kiosk (OS-level)](#ubuntu-kiosk-os-level) below)
- Windows: Assigned Access / kiosk mode
- macOS: device restrictions outside the app

## Current app modes

This repo currently supports two modes:

1. **Tauri desktop app** — real gateway WebSocket, pairing, live events.
2. **Local Python bridge** — `server.py` + browser for demos and webhook-triggered animations.

### Tauri desktop app

The desktop app loads the UI from `web/index.html`, stores settings locally, and connects to the real OpenClaw gateway WebSocket.

Settings include:

- **Setup code** (optional; encodes URL + bootstrap token)
- **Gateway URL** (`ws://` / `wss://`)
- **Display name** (pairing and gateway presence)
- **Screen title** (large heading at the top of the display)
- **Gateway token** (shared gateway auth token when not using only a setup code)

### Remote or LAN setup (optional)

Same flow as [Advanced: remote or LAN gateway](#advanced-remote-or-lan-gateway) at the top of this file—use the gateway’s URL, token or setup code, and approve pairing from the gateway host when prompted.

### Local Python bridge

The original local bridge is still available for testing animations and manual webhook events.

It serves the UI from `web/` and exposes:

- `GET /state`
- `POST /event`

Run:

```bash
./start.sh
```

Then open `http://localhost:8765`.

## Repository structure

- `web/index.html`: packaged display UI and connection logic
- `src-tauri/`: desktop shell and bundle configuration
- `server.py`: local development bridge and webhook receiver
- `start.sh`: convenience launcher for the local bridge

## Desktop development

You need:

- Node.js
- Rust toolchain (`cargo` must be on your `PATH` when you run `npm run build`)
- Tauri prerequisites for your OS

Install the official Tauri prerequisites before building, especially on Linux where system packages are required.

### Install Rust so `cargo` is available

If you see:

`failed to run command cargo metadata ... No such file or directory (os error 2)`

then `cargo` is missing from the environment `npm` uses. Install Rust with [rustup](https://rustup.rs/) (recommended):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then load Cargo into your **current** shell and confirm:

```bash
source "$HOME/.cargo/env"
command -v cargo
```

To make that permanent for new terminals, add this line to `~/.bashrc` or `~/.profile`:

```bash
source "$HOME/.cargo/env"
```

Build after a fresh login, or always run:

```bash
source "$HOME/.cargo/env" && npm run build
```

### Run and build

```bash
npm install
npm run dev
```

```bash
npm run build
```

## OpenClaw gateway integration

This app talks to OpenClaw using the built-in gateway protocol.

What it uses:

- gateway WebSocket handshake with `connect.challenge`
- device identity signing
- device pairing and paired device tokens
- gateway events such as `cron`, `heartbeat`, `chat`, and `sessions.changed`
- `sessions.subscribe` and `sessions.preview` for recent message activity

No custom backend endpoint is required beyond OpenClaw’s gateway.

## Local bridge development

Example webhook triggers (with `./start.sh` running):

```bash
curl -X POST http://localhost:8765/event \
  -H 'Content-Type: application/json' \
  -d '{
    "kind": "message",
    "source": "slack",
    "message": "New message in #ops",
    "duration_ms": 15000
  }'
```

```bash
curl -X POST http://localhost:8765/event \
  -H 'Content-Type: application/json' \
  -d '{
    "kind": "cron",
    "source": "nightly-sync",
    "message": "Running nightly sync",
    "duration_ms": 20000
  }'
```

## Installing the Linux build (Ubuntu, after `npm run build`)

Artifacts are under `src-tauri/target/release/bundle/`:

### Debian package (recommended on Ubuntu)

```bash
cd ~/openclaw-screensaver   # or your clone path
sudo apt install ./src-tauri/target/release/bundle/deb/*.deb
```

That pulls in declared dependencies. The binary is usually on `PATH` as:

```bash
openclaw-activity-screen
```

Confirm with:

```bash
dpkg -L openclaw-activity-screen | grep /bin/
```

Use that path in the autostart `Exec=` line below (often `/usr/bin/openclaw-activity-screen`).

### AppImage

```bash
chmod +x src-tauri/target/release/bundle/appimage/*.AppImage
./src-tauri/target/release/bundle/appimage/*.AppImage
```

For autostart, set `Exec=` to the **full path** to the AppImage.

## Ubuntu kiosk (OS-level)

For a public Ubuntu 24 (GNOME) display, treat the **session** as the security boundary: dedicated user, auto-login, autostart the app, and reduce escape hatches (lock screen, suspend, stray shortcuts). The Tauri window is already fullscreen without decorations; the OS should keep users in that session.

### 1. Dedicated user

Create a non-administrative user used only for the display (example name: `display`):

```bash
sudo adduser display
```

Install the `.deb` while logged in as an admin user (see above), then the `display` user can run `openclaw-activity-screen` from the menu or autostart.

### 2. Autostart the app (GNOME)

Log in once as `display`, then create an autostart entry. Use the real `Exec` path (installed `.deb` vs AppImage):

```bash
mkdir -p ~/.config/autostart
nano ~/.config/autostart/openclaw-activity-screen.desktop
```

Example when installed from the `.deb` (adjust if `dpkg -L` shows a different path):

```ini
[Desktop Entry]
Type=Application
Name=OpenClaw Activity Screen
Exec=/usr/bin/openclaw-activity-screen
X-GNOME-Autostart-enabled=true
```

Ensure OpenClaw itself is started for this machine if the screen should connect locally (systemd user service, login script, or another autostart entry — follow OpenClaw’s docs for your install).

### 3. Automatic login (GDM)

So the display boots straight into the `display` user:

- **GUI:** Settings → Users → unlock → enable **Automatic Login** for `display`.

- **Or** edit `/etc/gdm3/custom.conf` (Ubuntu with GDM) under `[daemon]`:

  ```ini
  AutomaticLoginEnable=true
  AutomaticLogin=display
  ```

Reboot and confirm the session opens without a password prompt.

### 4. Reduce screen lock and idle sleep (GNOME, as `display`)

```bash
gsettings set org.gnome.desktop.screensaver lock-enabled false
gsettings set org.gnome.desktop.session idle-delay 0
```

Optionally disable automatic suspend in **Settings → Power** for that user.

### 5. Hardening (short list)

- Do not grant `sudo` or unnecessary groups to the `display` user.
- Remove or hide installer shortcuts and terminal favorites if you want fewer escape routes.
- Prefer wired networking; firewall off services you do not need.
- For **stronger** kiosk isolation (single full-screen app only, no full desktop), consider a minimal stack (e.g. `cage` or `sway` launching only your app) or your organization’s standard kiosk image — that is separate from this repo.

The app is not a substitute for OS-level kiosk design.

## Current status

The app uses OpenClaw’s pairing and gateway event stream.

Remaining productization work includes polish, signed installers, and optional auto-start packaging.
