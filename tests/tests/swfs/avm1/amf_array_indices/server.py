from http.server import HTTPServer, BaseHTTPRequestHandler

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
        self.wfile.write(b"")

def run(server_class=HTTPServer, handler_class=MyHandler):
    server_address = ('', 8000)
    httpd = server_class(server_address, handler_class)
    print("Running server on port 8000...")
    httpd.serve_forever()

if __name__ == '__main__':
    run()
