package {
    import flash.display.MovieClip;
    import flash.events.StatusEvent;
    import flash.net.LocalConnection;

    public class Test extends MovieClip {
        public function Test() {
            var connection:LocalConnection = new LocalConnection();
            connection.addEventListener(StatusEvent.STATUS, function(event:StatusEvent):void {});

            check("send null connection", function():void {
                connection.send(null, "message");
            });
            check("send null method", function():void {
                connection.send("test", null);
            });
            check("send empty method", function():void {
                connection.send("test", "");
            });
            check("send empty connection + reserved method", function():void {
                connection.send("", "send");
            });
            check("send valid connection + reserved method", function():void {
                connection.send("test", "send");
            });
            check("send valid connection + valid method", function():void {
                connection.send("test", "message");
            });
            check("connect empty name", function():void {
                connection.connect("");
            });
            check("connect null name", function():void {
                connection.connect(null);
            });
            check("connect name containing colon", function():void {
                connection.connect("test:channel");
            });
            check("connect valid name", function():void {
                connection.connect("test_channel_" + new Date().time + "_" + Math.random());
                connection.close();
            });
        }

        private function check(name:String, operation:Function):void {
            trace("TEST: " + name);
            try {
                operation();
                trace("no error");
            } catch (error:Error) {
                trace(error.toString().replace(loaderInfo.url, "<SWF URL>"));
            }
            trace("END");
        }
    }
}
