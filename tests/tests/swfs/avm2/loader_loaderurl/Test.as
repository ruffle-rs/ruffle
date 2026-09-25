package {
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.net.URLRequest;

    public class Test extends MovieClip {
        private static var originalURL:String;

        public function Test() {
            var lastIndex = this.loaderInfo.url.lastIndexOf("/");
            Test.originalURL = this.loaderInfo.url.slice(0, lastIndex);

            trace("Test- url: " + Test.processURL(this.loaderInfo.url));
            trace("Test- loader url: " + Test.processURL(this.loaderInfo.loaderURL));

            var loader:Loader = new Loader();
            addChild(loader);
            loader.load(new URLRequest("load1.swf"));
        }

        public static function processURL(url:String) {
            return url.slice(Test.originalURL.length);
        }
    }
}
