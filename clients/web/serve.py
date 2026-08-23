#!/usr/bin/env python3
import http.server
import os
from pathlib import Path

PORT = 3001
DIST = Path(__file__).resolve().parent / "dist"


def main():
    os.chdir(DIST)
    handler = http.server.SimpleHTTPRequestHandler
    with http.server.HTTPServer(("", PORT), handler) as httpd:
        print(f"Serving clients/web/dist on http://localhost:{PORT}")
        httpd.serve_forever()


if __name__ == "__main__":
    main()
