from http.server import HTTPServer, BaseHTTPRequestHandler

img = None
with open("test.png", "rb") as f:
    img = f.read()

class MyHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        print("GET: " + self.path + " " + str(self.headers))

        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")

        resp_data = None
        if self.path == "/crossdomain.xml":
            resp_data = bytes("""
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE cross-domain-policy SYSTEM "http://www.adobe.com/xml/dtds/cross-domain-policy.dtd">
<cross-domain-policy>
  <allow-access-from domain="*" />
  <allow-http-request-headers-from domain="file://*" headers="*" secure="true"/>
</cross-domain-policy>
""", "utf-8")
            self.send_header("Content-type", "application/xml")
        else:
            resp_data = img
            self.send_header("Content-type", "image/png")

        self.end_headers()
        self.wfile.write(resp_data)

    def do_POST(self):
        print("POST: " + self.path + " " + str(self.headers))
        self.send_response(200)
        self.send_header("Content-type", "WTF-image/png")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(img)

def run(server_class=HTTPServer, handler_class=MyHandler):
    server_address = ('', 8000)
    httpd = server_class(server_address, handler_class)
    print("Running server")
    httpd.serve_forever()

run()
