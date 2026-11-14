package {
    import flash.display.MovieClip;
    import flash.net.*;
    import flash.system.fscommand;

    public class Test extends MovieClip {
        public function Test() {
            // allowNetworking="internal"

            check("navigateToURL", function():void {
                navigateToURL(new URLRequest("https://example.com/"), "_blank");
            });

            check("fscommand", function():void {
                fscommand("test", "argument");
            });

            // allowNetworking="none"

            check("sendToURL", function() {
                sendToURL(new URLRequest("https://example.com/"));
            });

            check("new URLLoader", function() {
                new URLLoader(new URLRequest("http://example1.com"));
            });

            check("URLLoader.load", function() {
                var loader = new URLLoader();
                loader.load(new URLRequest("http://example2.com"));
            });

            check("URLStream.load", function() {
                var stream = new URLStream();
                stream.load(new URLRequest("http://example3.com"));
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
