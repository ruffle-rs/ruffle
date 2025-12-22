package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.net.URLLoader;
    import flash.net.URLRequest;
    import flash.utils.setTimeout;

    public class Test extends Sprite {
        public function Test() {
            var req:URLRequest = new URLRequest("child.swf");
            var urlLoader:URLLoader = new URLLoader();
            urlLoader.dataFormat = "binary";
            urlLoader.addEventListener("complete", function(e) {
                var l:Loader = new Loader();

                l.contentLoaderInfo.addEventListener("init", function(e) {
                    trace("init event fired (this trace shouldn't appear!)");
                });
                l.contentLoaderInfo.addEventListener("complete", function(e) {
                    trace("complete event fired (this trace shouldn't appear!)");
                    trace(l.contentLoaderInfo.content);
                });

                l.loadBytes(urlLoader.data);
                addChild(l);

                trace(l.contentLoaderInfo.content);
            });
            urlLoader.load(req);
        }
    }
}
