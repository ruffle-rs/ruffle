from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from urllib.parse import urlsplit

FIXTURE_ROOT = Path(__file__).parent / "localhost"


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        request = self.rfile.read(int(self.headers.get("Content-Length", "0")))
        print()
        print("Navigator::fetch:")
        print(f"  URL: http://localhost:8000{self.path}")
        print("  Method: POST")
        print(f"  Mime-Type: {self.headers.get('Content-Type')}")
        print("  Body: [" + ", ".join(f"{byte:02X}" for byte in request) + "]")

        fixture_name = urlsplit(self.path).path.lstrip("/")
        fixture = (FIXTURE_ROOT / fixture_name).resolve()
        if FIXTURE_ROOT.resolve() not in fixture.parents or not fixture.is_file():
            self.send_error(404)
            return

        response = fixture.read_bytes()
        self.send_response(200)
        self.send_header("Content-Type", "application/x-amf")
        self.send_header("Content-Length", str(len(response)))
        self.end_headers()
        self.wfile.write(response)

    def log_message(self, format, *args):
        print(format % args)


if __name__ == "__main__":
    print("Serving AMF0 fixture on http://localhost:8000/test2")
    HTTPServer(("", 8000), Handler).serve_forever()
