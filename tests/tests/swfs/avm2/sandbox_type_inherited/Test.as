package {
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.net.URLLoader;
    import flash.net.URLRequest;
    import flash.system.Security;

    public class Test extends MovieClip {
        public function Test() {
            trace("This SWF is of sandbox type " + Security.sandboxType);

            var urlLoader:URLLoader = new URLLoader();
            urlLoader.dataFormat = "binary";
            urlLoader.addEventListener("complete", function(e) {
                var loader:Loader = new Loader();
                loader.loadBytes(urlLoader.data);
            });
            urlLoader.load(new URLRequest("loaded.swf"));
        }
    }
}
