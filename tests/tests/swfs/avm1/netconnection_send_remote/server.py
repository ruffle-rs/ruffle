from http.server import HTTPServer, BaseHTTPRequestHandler

responses = [
    #     let packet = Packet {
    #         version: AMFVersion::AMF0,
    #         headers: vec![],
    #         messages: vec![Message{
    #             target_uri: "/1/onStatus".to_string(),
    #             response_uri: "".to_string(),
    #             contents: Rc::new(Value::String("Success!".to_string())),
    #         }],
    #     };
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x0B, 0x2F, 0x31, 0x2F, 0x6F, 0x6E, 0x52, 0x65, 0x73, 0x75, 0x6C, 0x74,
     0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x02, 0x00, 0x08, 0x53, 0x75, 0x63, 0x63, 0x65, 0x73, 0x73, 0x21],

    #     let packet = Packet {
    #         version: AMFVersion::AMF0,
    #         headers: vec![],
    #         messages: vec![Message{
    #             target_uri: "/1/onStatus".to_string(),
    #             response_uri: "".to_string(),
    #             contents: Rc::new(Value::Number(-123.0)),
    #         }],
    #     };
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x0B, 0x2F, 0x31, 0x2F, 0x6F, 0x6E, 0x53, 0x74, 0x61, 0x74, 0x75, 0x73,
     0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0xC0, 0x5E, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00],

    #     let packet = Packet {
    #         version: AMFVersion::AMF0,
    #         headers: vec![Header {
    #             name: "Response Header".to_string(),
    #             must_understand: false,
    #             value: Rc::new(Value::String("Spookily ignored by flash!".to_string())),
    #         }],
    #         messages: vec![
    #             Message {
    #                 target_uri: "/1/onStatus".to_string(),
    #                 response_uri: "".to_string(),
    #                 contents: Rc::new(Value::Bool(false)),
    #             },
    #             Message {
    #                 target_uri: "/2/onResult".to_string(),
    #                 response_uri: "".to_string(),
    #                 contents: Rc::new(Value::Bool(true)),
    #             },
    #         ],
    #     };
    [0x00, 0x00, 0x00, 0x01, 0x00, 0x0F, 0x52, 0x65, 0x73, 0x70, 0x6F, 0x6E, 0x73, 0x65, 0x20, 0x48, 0x65, 0x61, 0x64,
     0x65, 0x72, 0x00, 0x00, 0x00, 0x00, 0x1D, 0x02, 0x00, 0x1A, 0x53, 0x70, 0x6F, 0x6F, 0x6B, 0x69, 0x6C, 0x79, 0x20,
     0x69, 0x67, 0x6E, 0x6F, 0x72, 0x65, 0x64, 0x20, 0x62, 0x79, 0x20, 0x66, 0x6C, 0x61, 0x73, 0x68, 0x21, 0x00, 0x02,
     0x00, 0x0B, 0x2F, 0x31, 0x2F, 0x6F, 0x6E, 0x53, 0x74, 0x61, 0x74, 0x75, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
     0x01, 0x00, 0x00, 0x0B, 0x2F, 0x32, 0x2F, 0x6F, 0x6E, 0x52, 0x65, 0x73, 0x75, 0x6C, 0x74, 0x00, 0x00, 0x00, 0x00,
     0x00, 0x02, 0x01, 0x01],

    # Give a 404 error
    None,
]

class MyHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        print("")
        print("Navigator::fetch:")
        print("  URL: http://localhost:8000")
        print("  Method: POST")
        print(f"  Mime-Type: {self.headers.get('Content-Type')}")

        request = self.rfile.read(int(self.headers['Content-Length']))
        request = ", ".join([f"{byte:02X}" for byte in request])

        print(f"  Body: [{request}]")
        print("")

        response = responses.pop(0)
        if response:
            self.send_response(200)
            self.end_headers()
            self.wfile.write(bytes(response))
        else:
            self.send_response(404)

def run(server_class=HTTPServer, handler_class=MyHandler):
    server_address = ('', 8000)
    httpd = server_class(server_address, handler_class)
    print("Running server")
    httpd.serve_forever()

run()
