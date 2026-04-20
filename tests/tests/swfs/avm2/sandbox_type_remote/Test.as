package {
    import flash.display.MovieClip;
    import flash.display.Loader;
    import flash.system.Security;
    import flash.net.URLRequest;

    // Compile with network enabled
    public class Test extends MovieClip {
        public function Test() {
            trace("Current sandbox type: " + Security.sandboxType);

            var test:Test = this;
            test.loadSwf("http://localhost:8000/test-network.swf", function():void {
                test.loadSwf("http://localhost:8000/test-no-network.swf", function():void {});
            });
        }

        private function loadSwf(url:String, callback:Function):void {
            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener("complete", callback);
            loader.load(new URLRequest(url));
        }
    }
}
