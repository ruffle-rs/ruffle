class Test {
    static function main() {
        var socket = new XMLSocket();

        socket.onConnect = function (status:Boolean) {
            trace("connected");
            trace(status);
        };

        socket.onData = function (data:String) {
            trace("data:")
            trace(data);
            trace("Closing socket in onData handler");
            socket.close();
            trace("Successfully closed socket");
        };
        socket.onClose = function () {
            trace("closed");
        };

        socket.connect("localhost", 8001);
    }
}
