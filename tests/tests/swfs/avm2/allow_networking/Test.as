package {
    import flash.display.MovieClip;
    import flash.net.*;
    import flash.system.fscommand;
    import flash.external.ExternalInterface;

    public class Test extends MovieClip {
        public function Test() {
            // allowNetworking="internal"

            check("navigateToURL", function():void {
                navigateToURL(new URLRequest("https://example.com/"), "_blank");
            });

            check("fscommand", function():void {
                fscommand("test", "argument");
            });

            check("ExternalInterface.available", function():void {
                trace("value: " + ExternalInterface.available);
            });

            check("ExternalInterface.call", function():void {
                ExternalInterface.call("foo");
            });

            check("ExternalInterface.addCallback", function():void {
                ExternalInterface.addCallback("callme", function() {})
            });

            // allowNetworking="none"

            check("sendToURL", function() {
                sendToURL(new URLRequest("https://example.com/"));
            });
        }

        private function check(name:String, f:Function):void {
            trace("TEST: " + name);
            try {
                f();
                trace("no error");
            } catch (error:*) {
                trace(error.constructor);
                trace(error.getStackTrace());
            }
            trace("END");
        }
    }
}
