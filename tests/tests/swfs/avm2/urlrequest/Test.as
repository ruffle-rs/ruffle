package {
    import flash.display.Sprite;
    public class Test extends Sprite {
    }
}

import flash.net.*;

var request = new URLRequest();

request.url = null;

var methods = ["GET", "get", "Get", "POST", "post", "PoST", "PUT", "put", "DELETE"];
for each (var method in methods) {
    trace("// '" + method + "' method");
    try {
        request.method = method;
        trace("request.method: " + request.method);
    } catch (e) {
        trace("error: " + e);
    }
}
