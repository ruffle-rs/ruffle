from http.server import HTTPServer, BaseHTTPRequestHandler


# AMF0 response: version 0, no headers, one /1/onResult message whose body
# consists solely of the AMF0 Unsupported marker (0x0D).
RESPONSE = (
    b"\x00\x00"                    # AMF version
    b"\x00\x00"                    # header count
    b"\x00\x01"                    # message count
    b"\x00\x0b/1/onResult"        # target URI
    b"\x00\x00"                    # response URI
    b"\x00\x00\x00\x01"          # body length
    b"\x0d"                        # Unsupported
)

class MyHandler(BaseHTTPRequestHandler):
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

        self.send_response(200)
        self.end_headers()
        self.wfile.write(RESPONSE)

    def log_message(self, format, *args):
        print(format % args)

def run(server_class=HTTPServer, handler_class=MyHandler):
    server_address = ('', 8000)
    httpd = server_class(server_address, handler_class)
    print("Running server on port 8000...")
    httpd.serve_forever()

if __name__ == '__main__':
    run()
