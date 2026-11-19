from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path

RESPONSE_PATH = Path(__file__).parent / "localhost" / "test1"


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        print("")
        print("Navigator::fetch:")
        print(f"  URL: http://localhost:8000{self.path}")
        print("  Method: POST")
        print(f"  Mime-Type: {self.headers.get('Content-Type')}")

        request = self.rfile.read(int(self.headers['Content-Length']))
        # Format as uppercase hex bytes
        request_hex = ", ".join([f"{byte:02X}" for byte in request])

        print(f"  Body: [{request_hex}]")
        print("")

        response = RESPONSE_PATH.read_bytes()
        self.send_response(200)
        self.end_headers()
        self.wfile.write(response)

    def log_message(self, format, *args):
        print(format % args)


if __name__ == "__main__":
    print("Serving AMF0 StrictArray response on http://localhost:8000/test1")
    HTTPServer(("", 8000), Handler).serve_forever()
