package {
    import flash.display.MovieClip;
    import flash.net.navigateToURL;
    import flash.net.URLRequest;
    import flash.system.fscommand;

    public class Test extends MovieClip {
        public function Test() {
            check("navigateToURL", function():void {
                navigateToURL(new URLRequest("https://example.com/"), "_blank");
            });

            check("fscommand", function():void {
                fscommand("test", "argument");
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
