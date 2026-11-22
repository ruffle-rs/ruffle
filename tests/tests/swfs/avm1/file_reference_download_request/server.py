"""Server for comparing the downloads of test.swf in Flash Player and Ruffle.

Serves the files in `localhost` in chunks of the size given by the
`debug-chunk-size` query parameter, and without a `Content-Length` with
`debug-no-content-length` (like the test runner does), and saves every
request it receives to the `requests` directory.
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

        path = os.path.join(HERE, "localhost", url.path.lstrip("/"))
        if not os.path.isfile(path):
            return self.reply(b"", "text/plain", None, 404)

        query = urllib.parse.parse_qs(url.query, keep_blank_values=True)
        chunk_size = int(query.get("debug-chunk-size", ["0"])[0]) or None
        has_length = "debug-no-content-length" not in query
        with open(path, "rb") as f:
            self.reply(f.read(), "text/plain", chunk_size, has_length=has_length)

    def reply(self, body, content_type, chunk_size, status=200, has_length=True):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        if has_length:
            self.send_header("Content-Length", str(len(body)))
        else:
            self.send_header("Transfer-Encoding", "chunked")
        self.end_headers()
        step = chunk_size or len(body) or 1
        for i in range(0, len(body), step):
            chunk = body[i : i + step]
            if not has_length:
                chunk = b"%x\r\n" % len(chunk) + chunk + b"\r\n"
            self.wfile.write(chunk)
            self.wfile.flush()
            if chunk_size:
                time.sleep(0.1)
        if not has_length:
            self.wfile.write(b"0\r\n\r\n")
            self.wfile.flush()

    do_GET = handle_request
    do_POST = handle_request


if __name__ == "__main__":
    http.server.ThreadingHTTPServer(("localhost", 8000), Handler).serve_forever()
