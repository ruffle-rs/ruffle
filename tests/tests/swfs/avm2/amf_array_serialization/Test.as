package {
    import flash.display.Sprite;
    import flash.net.NetConnection;
    import flash.net.Responder;
    import flash.net.LocalConnection;
    import flash.utils.ByteArray;

    public class Test extends Sprite {
        public function Test() {
            
            // --- 1. SETUP THE DATA STRUCTURES ---
            var as3Dense:Array = ["dense_0", "dense_1"];
            
            var as3Sparse:Array = [];
            as3Sparse[0] = "sparse_0";
            as3Sparse[5] = "sparse_5";

            var as3Mixed:Array = ["mixed_0"];
            as3Mixed["custom_prop"] = "custom_value";

            var as3Fake:Object = { "0": "fake_0", "length": 1 };

            var as3Date:Date = new Date(1672531200000);

            // NEW: Nested Object
            var as3Nested:Array = [["deep_0", "deep_1"]];


            // --- 2. TEST BYTEARRAY (AMF0 & AMF3 Memory Boundaries) ---
            trace("--- Testing ByteArray AMF0 ---");
            var ba0:ByteArray = new ByteArray();
            ba0.objectEncoding = 0; 
            ba0.writeObject(as3Dense);
            ba0.writeObject(as3Sparse);
            ba0.writeObject(as3Mixed);
            ba0.writeObject(as3Fake);
            ba0.writeObject(as3Date);
            ba0.writeObject(as3Nested);
            
            ba0.position = 0;
            var readBa0:* = ba0.readObject();
            trace("ByteArray AMF0 Read: " + readBa0[0]);

            trace("--- Testing ByteArray AMF3 ---");
            var ba3:ByteArray = new ByteArray();
            ba3.objectEncoding = 3; 
            ba3.writeObject(as3Dense);
            ba3.writeObject(as3Sparse);
            ba3.writeObject(as3Mixed);
            ba3.writeObject(as3Fake);
            ba3.writeObject(as3Date);
            ba3.writeObject(as3Nested);
            
            ba3.position = 0;
            var readBa3:* = ba3.readObject();


            // --- 3. TEST LOCALCONNECTION (AMF0 Wire Boundaries) ---
            trace("--- Testing LocalConnection ---");
            var lcReceiver:LocalConnection = new LocalConnection();
            lcReceiver.client = {
                onReceiveArrays: function(d:*, s:*, m:*, f:*, date:*, n:*):void {
                    trace("LC Received: " + date.getTime());
                }
            };
            try { lcReceiver.connect("amf3_test_connection"); } catch (e:Error) {}

            var lcSender:LocalConnection = new LocalConnection();
            lcSender.send("amf3_test_connection", "onReceiveArrays", as3Dense, as3Sparse, as3Mixed, as3Fake, as3Date, as3Nested);


            // --- 4. TEST NETCONNECTION (AMF0 & AMF3 Wire Paths) ---
            var responder:Responder = new Responder(
                function(res:Object):void { trace("NC Success"); },
                function(err:Object):void { trace("NC Failed"); }
            );

            trace("--- Testing NetConnection AMF0 ---");
            var nc0:NetConnection = new NetConnection();
            nc0.objectEncoding = 0; 
            nc0.connect("http://localhost:8000/");
            nc0.call("test.avm2.amf0", responder, as3Dense, as3Sparse, as3Mixed, as3Fake, as3Date, as3Nested);

            trace("--- Testing NetConnection AMF3 ---");
            var nc3:NetConnection = new NetConnection();
            nc3.objectEncoding = 3; 
            nc3.connect("http://localhost:8000/");
            nc3.call("test.avm2.amf3", responder, as3Dense, as3Sparse, as3Mixed, as3Fake, as3Date, as3Nested);
        }
    }
}
