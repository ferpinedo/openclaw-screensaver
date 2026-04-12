# OpenClaw Screensaver

Browser-based activity screen for OpenClaw.

It runs as a small local web app with two visual states:

- `idle`: calm ambient animation while nothing is happening
- `busy`: higher-energy animation when OpenClaw receives messages or runs jobs

The screen is meant to stay open full-screen on a monitor, TV, or browser window and react to webhook events.

## Requirements

- Python 3
- A modern browser

## Run locally

```bash
cd openclaw-screensaver
./start.sh
```

Then open:

```bash
http://localhost:8765
```

For the intended effect, keep the page in full-screen mode.

## Trigger the busy state

Send a `POST` request to `http://localhost:8765/event`.

Example for a message event:

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

Example for a scheduled job event:

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

## Event payload

```json
{
  "kind": "message",
  "source": "slack",
  "message": "New message in support",
  "duration_ms": 15000
}
```

Fields:

- `kind`: event type such as `message` or `cron`
- `source`: where the event came from
- `message`: text shown on screen
- `duration_ms`: how long the screen should stay in busy mode

## Suggested OpenClaw integration

Have OpenClaw send a `POST` to `http://localhost:8765/event` whenever it:

- receives a message from any channel
- starts a cron job
- finishes a cron job, if you want to surface activity again

## Project structure

- `index.html`: front-end screensaver UI and animation
- `server.py`: lightweight local HTTP server and event endpoint
- `start.sh`: convenience script to launch the app

## Display tips

- macOS: open it in Chrome, Arc, or Safari and use full-screen mode
- Linux: Chromium kiosk mode works well with `--kiosk http://localhost:8765`
- Windows: Edge or Chrome app/kiosk mode works well

## Possible next step

If you want to turn this into a native desktop app later, a good next step would be wrapping it in Electron or Tauri and listening for local events via webhook or WebSocket.
