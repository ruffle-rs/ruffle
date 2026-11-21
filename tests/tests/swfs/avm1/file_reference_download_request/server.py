"""Server for comparing the downloads of test.swf in Flash Player and Ruffle.

Serves `localhost/download.txt` in chunks of the size given by the
`debug-chunk-size` query parameter (like the test runner does), and saves
every request it receives to the `requests` directory.
"""

import http.server
import itertools
import os
import time
import urllib.parse

HERE = os.path.dirname(os.path.abspath(__file__))
REQUESTS = os.path.join(HERE, "requests")
counter = itertools.count(1)


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def handle_request(self):
        body = self.rfile.read(int(self.headers.get("Content-Length") or 0))
        url = urllib.parse.urlsplit(self.path)

        # Serving the SWF lets Flash Player send requests to this server
        # without a trust file.
        if url.path == "/test.swf":
            with open(os.path.join(HERE, "test.swf"), "rb") as f:
                return self.reply(f.read(), "application/x-shockwave-flash", None)

        os.makedirs(REQUESTS, exist_ok=True)
        name = os.path.join(REQUESTS, "%03d.txt" % next(counter))
        with open(name, "wb") as f:
            f.write(self.requestline.encode() + b"\r\n")
            f.write(str(self.headers).encode())
            f.write(body)
        print("Saved", self.requestline, "to", name)

        if url.path != "/download.txt":
            return self.reply(b"", "text/plain", None, 404)

        query = urllib.parse.parse_qs(url.query)
        chunk_size = int(query.get("debug-chunk-size", ["0"])[0]) or None
        with open(os.path.join(HERE, "localhost", "download.txt"), "rb") as f:
            self.reply(f.read(), "text/plain", chunk_size)

    def reply(self, body, content_type, chunk_size, status=200):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        for i in range(0, len(body), chunk_size or len(body) or 1):
            self.wfile.write(body[i : i + (chunk_size or len(body))])
            self.wfile.flush()
            if chunk_size:
                time.sleep(0.1)

    do_GET = handle_request
    do_POST = handle_request


if __name__ == "__main__":
    http.server.ThreadingHTTPServer(("localhost", 8000), Handler).serve_forever()
