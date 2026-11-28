"""Server for comparing the requests of test.swf in Flash Player and Ruffle.

Answers like the test runner's fake responses do, depending on the URL's
query, and saves every request it receives to the `requests` directory.
"""

import http.server
import itertools
import os
import urllib.parse

HERE = os.path.dirname(os.path.abspath(__file__))
REQUESTS = os.path.join(HERE, "requests")
counter = itertools.count(1)


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def reply(self, status, body, content_type="text/plain"):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def save(self, body):
        os.makedirs(REQUESTS, exist_ok=True)
        name = os.path.join(REQUESTS, "%03d.txt" % next(counter))
        with open(name, "wb") as f:
            f.write(self.requestline.encode() + b"\r\n")
            f.write(str(self.headers).encode())
            f.write(body)
        print("Saved", self.requestline, "to", name)

    def respond(self):
        query = urllib.parse.urlsplit(self.path).query
        if query.startswith("debug-success"):
            self.reply(200, b"Hello, World!")
        else:
            self.reply(404, b"")

    def do_GET(self):
        # Serving the SWF lets Flash Player send requests to this server
        # without a trust file.
        if self.path == "/test.swf":
            with open(os.path.join(HERE, "test.swf"), "rb") as f:
                return self.reply(200, f.read(), "application/x-shockwave-flash")
        self.save(b"")
        self.respond()

    def do_POST(self):
        self.save(self.rfile.read(int(self.headers.get("Content-Length") or 0)))
        self.respond()


if __name__ == "__main__":
    http.server.ThreadingHTTPServer(("localhost", 8000), Handler).serve_forever()
