#!/usr/bin/env python3
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
import json
import time

ROOT = Path(__file__).resolve().parent
STATE = {
    "kind": "idle",
    "source": "system",
    "message": "Esperando actividad…",
    "duration_ms": 15000,
    "last_event_ts": None,
}


class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ROOT), **kwargs)

    def _send_json(self, payload, code=200):
        data = json.dumps(payload).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        if self.path == "/state":
            return self._send_json(STATE)
        return super().do_GET()

    def do_POST(self):
        if self.path != "/event":
            return self._send_json({"ok": False, "error": "Not found"}, 404)
        length = int(self.headers.get("Content-Length", "0"))
        raw = self.rfile.read(length) if length else b"{}"
        try:
            payload = json.loads(raw.decode("utf-8"))
        except Exception:
            payload = {}

        STATE.update({
            "kind": payload.get("kind", "message"),
            "source": payload.get("source", "unknown"),
            "message": payload.get("message", "Actividad detectada"),
            "duration_ms": int(payload.get("duration_ms", 15000)),
            "last_event_ts": int(time.time() * 1000),
        })
        return self._send_json({"ok": True, "state": STATE})


def main():
    port = 8765
    server = ThreadingHTTPServer(("0.0.0.0", port), Handler)
    print(f"OpenClaw Activity Screen disponible en http://localhost:{port}")
    print("POST /event con JSON para disparar el modo busy")
    server.serve_forever()


if __name__ == "__main__":
    main()
