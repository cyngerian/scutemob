#!/usr/bin/env python3
"""Serve the mockups with caching disabled, so an edited common.css / common.js is
never stale in the owner's browser. Usage: python3 serve.py [port=3043]."""
import sys, http.server, functools
class H(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cache-Control', 'no-store'); super().end_headers()
    def log_message(self, *a): pass
port = int(sys.argv[1]) if len(sys.argv) > 1 else 3043
http.server.ThreadingHTTPServer(('127.0.0.1', port), functools.partial(H, directory=__import__('os').path.dirname(__file__) or '.')).serve_forever()
