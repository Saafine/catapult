#!/usr/bin/env python3
"""Serve web/ without caching, so a reload always shows the latest build.
usage: python3 tools/serve.py [port]   (default 8080)"""
import http.server, os, sys

class NoCache(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0")
        self.send_header("Pragma", "no-cache")
        self.send_header("Expires", "0")
        super().end_headers()

    def log_message(self, *args):
        pass

port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "web"))
print(f"serving web/ on http://localhost:{port} (no cache)")
http.server.ThreadingHTTPServer(("", port), NoCache).serve_forever()
