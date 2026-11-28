import struct
from http.server import BaseHTTPRequestHandler, HTTPServer

XML_CONTENT = b"<a></b>"


def response_packet():
    target_uri = b"/1/onResult"
    amf_value = b"\x0f" + struct.pack(">I", len(XML_CONTENT)) + XML_CONTENT

    message = (
        struct.pack(">H", len(target_uri))
        + target_uri
        + struct.pack(">H", 0)  # Empty response URI
        + struct.pack(">I", len(amf_value))
        + amf_value
    )

    return (
        struct.pack(">HHH", 0, 0, 1)
        + message
    )


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        request = self.rfile.read(int(self.headers.get("Content-Length", "0")))

        print()
        print("Navigator::fetch:")
        print(f"  URL: http://localhost:8000{self.path}")
        print("  Method: POST")
        print(f"  Mime-Type: {self.headers.get('Content-Type')}")
        print("  Body: [" + ", ".join(f"{byte:02X}" for byte in request) + "]")

        response = response_packet()
        self.send_response(200)
        self.send_header("Content-Type", "application/x-amf")
        self.send_header("Content-Length", str(len(response)))
        self.end_headers()
        self.wfile.write(response)

    def log_message(self, format, *args):
        print(format % args)


if __name__ == "__main__":
    print("Serving malformed AMF0 XML on http://localhost:8000/test1")
    HTTPServer(("", 8000), Handler).serve_forever()
