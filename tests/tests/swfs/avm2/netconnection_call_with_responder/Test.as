package {
    import flash.display.Sprite;
    import flash.net.NetConnection;
    import flash.net.ObjectEncoding;
    import flash.net.Responder;

    public class Test extends Sprite {
        public function Test() {
            var connection:NetConnection = new NetConnection();
            connection.objectEncoding = ObjectEncoding.AMF0;
            connection.connect("http://localhost:8000/");

            // Use a Responder so the test focuses on AMF serialization rather
            // than the separate response-URI behavior for fire-and-forget calls.
            var responder:Responder = new Responder(
                function(result:Object):void {},
                function(status:Object):void {}
            );

            connection.call("test.withResponder", responder, {
                value: "sent with a responder"
            });
            trace("NetConnection.call invoked with a Responder");
        }
    }
}
