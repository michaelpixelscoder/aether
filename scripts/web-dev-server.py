#!/usr/bin/env python3
"""Static development server with browser reload notifications for dist/.aether-reload."""

from __future__ import annotations

import argparse
import os
import time
from http import HTTPStatus
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


RELOAD_PATH = "/__aether_live_reload__"
RELOAD_SNIPPET = b"""<script>
new EventSource('/__aether_live_reload__').onmessage = () => location.reload();
</script>"""


class DevelopmentHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, directory: str, **kwargs):
        self.root = Path(directory)
        super().__init__(*args, directory=directory, **kwargs)

    def do_GET(self) -> None:
        if self.path.split("?", 1)[0] == RELOAD_PATH:
            self.serve_reload_events()
            return
        if self.serve_html_with_reload():
            return
        super().do_GET()

    def end_headers(self) -> None:
        # A reload must fetch the freshly emitted JavaScript and WASM artifacts.
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def serve_html_with_reload(self) -> bool:
        path = Path(self.translate_path(self.path))
        if path.is_dir():
            path /= "index.html"
        if path.suffix != ".html" or not path.is_file():
            return False

        body = path.read_bytes().replace(b"</body>", RELOAD_SNIPPET + b"</body>")
        self.send_response(HTTPStatus.OK)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
        return True

    def serve_reload_events(self) -> None:
        marker = self.root / ".aether-reload"
        try:
            initial_mtime = marker.stat().st_mtime_ns
        except FileNotFoundError:
            initial_mtime = 0

        self.send_response(HTTPStatus.OK)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Connection", "close")
        self.end_headers()

        try:
            while True:
                time.sleep(0.25)
                try:
                    changed = marker.stat().st_mtime_ns != initial_mtime
                except FileNotFoundError:
                    changed = initial_mtime != 0
                if changed:
                    self.wfile.write(b"data: reload\n\n")
                    self.wfile.flush()
                    self.close_connection = True
                    return
        except (BrokenPipeError, ConnectionResetError):
            return


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", required=True)
    parser.add_argument("--port", type=int, default=8080)
    args = parser.parse_args()

    handler = lambda *handler_args, **handler_kwargs: DevelopmentHandler(
        *handler_args, directory=os.path.abspath(args.directory), **handler_kwargs
    )
    server = ThreadingHTTPServer(("", args.port), handler)
    print(f"Development server listening on http://localhost:{args.port}/", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
