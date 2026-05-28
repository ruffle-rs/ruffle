package {
    import flash.display.MovieClip;
    import flash.net.NetConnection;
    import flash.net.ObjectEncoding;
    import flash.net.Responder;

    public class Test extends MovieClip {
        public function Test() {
            var nc:NetConnection = new NetConnection();

            // This test targets the AMF0 wire format. At time of writing Ruffle
            // doesn't support AMF3 properly, and the StrictArray fix (#16381) is
            // on the AMF0 path, so pin the encoding to AMF0 like the sibling
            // netconnection_send_remote test does.
            nc.objectEncoding = ObjectEncoding.AMF0;
            nc.connect("http://localhost:8000/");

            var responder:Responder = new Responder(onResult, null);

            // A genuine dense AS3 Array. Real Flash sends this as an AMF0
            // StrictArray (marker 0x0A); before #16381 Ruffle emitted an
            // ECMAArray (marker 0x08) with "0"/"1" string keys instead.
            var realArray:Array = new Array();
            realArray[0] = "real_0";
            realArray[1] = "real_1";

            trace("--- Testing NetConnection Array Serialization ---");
            nc.call("test.arrays", responder, realArray);
        }

        private function onResult(result:*):void {
            trace("Received result");
        }
    }
}
