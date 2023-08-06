class Test {
    static function main() {
        var socket = new XMLSocket();

        trace("Check for socket errors");
        socket.close();
        socket.send("Hello!");

        socket.onConnect = function(status:Boolean) {
            trace("connected status:");
            trace(status);

            if (status) {
                socket.send("Hello!");
                socket.send(new XML("<root><item></item></root>"));
            }
        };

        socket.onXML = function(data:XML) {
            trace(data);
        };

        socket.onClose = function() {
            trace("closed");
        };

        socket.connect("localhost", 8001);
    }
}
