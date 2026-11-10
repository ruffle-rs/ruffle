from http.server import HTTPServer, BaseHTTPRequestHandler


class MyHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        request = self.rfile.read(int(self.headers["Content-Length"]))
        request_hex = ", ".join(f"{byte:02X}" for byte in request)

        print()
        print("Navigator::fetch:")
        print("  URL: http://localhost:8000/")
        print("  Method: POST")
        print(f"  Mime-Type: {self.headers.get('Content-Type')}")
        print(f"  Body: [{request_hex}]")

        # AMF0 packet: one null result for the first call's /1/onResult URI.
        response = (
            bytes([0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x0B])
            + b"/1/onResult"
            + bytes([0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x05])
        )

        self.send_response(200)
        self.send_header("Content-Type", "application/x-amf")
        self.send_header("Content-Length", str(len(response)))
        self.end_headers()
        self.wfile.write(response)


def run():
    server = HTTPServer(("", 8000), MyHandler)
    print("Listening on port 8000...")
    server.serve_forever()


if __name__ == "__main__":
    run()
