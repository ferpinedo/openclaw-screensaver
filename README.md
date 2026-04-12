# OpenClaw Activity Screen

Public activity display for OpenClaw.

This project is moving from a local Python + browser demo into a packaged desktop app built with Tauri so users can install it, pair it with OpenClaw, and leave it running full-screen in a public space.

## Product direction

The intended end-user flow is:

1. Download an installer from GitHub Releases.
2. Open the app.
3. Paste an OpenClaw setup code or gateway URL.
4. Leave the app running full-screen while OpenClaw pushes events in real time.

The public-machine lock down should happen at the OS level, not inside the app.

- Linux: use a dedicated kiosk session or locked-down desktop account
- Windows: use Assigned Access / kiosk mode
- macOS: use device restrictions outside the app

## Current app modes

This repo currently supports two modes:

1. `Tauri desktop app`
2. `Local Python bridge` for development and compatibility with the original webhook demo

### Tauri desktop app

The desktop app loads the UI directly from `web/index.html`, stores settings locally, and connects to the real OpenClaw gateway WebSocket.

Current first-run fields:

- `Setup code`
- `Gateway URL`
- `Display name`
- `Gateway token`

Open settings at any time with `Ctrl+,`.

Recommended setup path:

1. Generate a setup code from the OpenClaw machine:

```bash
openclaw qr --setup-code-only
```

2. Paste that code into the app.
3. Approve the pending pairing request:

```bash
openclaw devices list
openclaw devices approve <requestId>
```

After the first successful connect, the screen stores its paired device token and reconnects with that bounded token automatically.

### Same-machine setup

If OpenClaw and the screen app run on the same computer, you usually do not need a setup code.

Use the in-app `Use local OpenClaw` button, which pre-fills:

```text
ws://127.0.0.1:18789
```

Then paste your gateway token and connect.

You can read the token with:

```bash
openclaw config get gateway.auth.token
```

The first local connection still creates a pairing request that you approve once:

```bash
openclaw devices list
openclaw devices approve <requestId>
```

After that first approval, the app reconnects locally using its stored device token.

### Local Python bridge

The original local bridge is still available for testing animations and manual webhook events.

It serves the UI from `web/` and exposes:

- `GET /state`
- `POST /event`

## Repository structure

- `web/index.html`: packaged display UI and connection logic
- `src-tauri/`: desktop shell and bundle configuration
- `server.py`: local development bridge and webhook receiver
- `start.sh`: convenience launcher for the local bridge

## Desktop development

You need:

- Node.js
- Rust toolchain
- Tauri prerequisites for your OS

Install the official Tauri prerequisites before building, especially on Linux where system packages are required.

Install dependencies:

```bash
npm install
```

Run the desktop app in development:

```bash
npm run dev
```

Build installers for the current platform:

```bash
npm run build
```

## OpenClaw gateway integration

This app now talks to OpenClaw using the built-in gateway protocol instead of a custom display-only endpoint.

What it uses:

- gateway WebSocket handshake with `connect.challenge`
- device identity signing
- device pairing and paired device tokens
- gateway events such as `cron`, `heartbeat`, `chat`, and `sessions.changed`
- `sessions.subscribe` and `sessions.preview` to surface recent message activity

The intended OpenClaw-side setup is already available in OpenClaw itself. No custom backend endpoint is required for this app.

For remote/public installs, prefer secure gateway URLs:

- `wss://...` for internet or public-network access
- `ws://...` only for localhost or trusted private LAN use

The easiest operator flow is:

```bash
openclaw qr --setup-code-only
openclaw devices list
openclaw devices approve <requestId>
```

## Local bridge development

Run the original Python server:

```bash
./start.sh
```

Then open:

```bash
http://localhost:8765
```

Trigger a message event:

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

Trigger a cron event:

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

## Linux kiosk deployment

For a public installation on Linux, the recommended split is:

1. Tauri app for the display experience
2. Linux kiosk session for machine lockdown

Recommended deployment rules:

- dedicated user account for the display
- auto-login into that account
- auto-start the app on login
- restrict shell access and desktop shortcuts
- disable unnecessary services and notifications
- require admin credentials to leave kiosk mode

The app should not be treated as the security boundary. The OS session is the security boundary.

## Current status

The app now uses OpenClaw's existing pairing and gateway event stream.

The remaining work in this repo is mostly productization:

1. polish the pairing UX further
2. package signed installers for Linux, Windows, and macOS
3. add auto-start and kiosk-oriented deployment guidance
