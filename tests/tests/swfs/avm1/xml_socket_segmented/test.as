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
        };

        socket.onXML = function (src:XML) {
            trace("This should never run.");
        };

        socket.onClose = function () {
            trace("closed");
        };

        socket.connect("localhost", 8001);
    }
}
