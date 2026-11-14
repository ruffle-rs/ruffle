package {
    import flash.display.MovieClip;
    import flash.net.*;
    import flash.system.fscommand;

    public class Test extends MovieClip {
        public function Test() {
            check("navigateToURL", function():void {
                navigateToURL(new URLRequest("https://example.com/"), "_blank");
            });

            check("fscommand", function():void {
                fscommand("test", "argument");
            });

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
