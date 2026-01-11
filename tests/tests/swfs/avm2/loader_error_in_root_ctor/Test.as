package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.net.URLRequest;
    import flash.utils.setTimeout;

    public class Test extends Sprite {
        public function Test() {
            var req:URLRequest = new URLRequest("child.swf");
            var l:Loader = new Loader();

            l.contentLoaderInfo.addEventListener("init", function(e) {
                trace("init event fired (this trace shouldn't appear!)");
            });
            l.contentLoaderInfo.addEventListener("complete", function(e) {
                trace("complete event fired (this trace shouldn't appear!)");
            });

            l.load(req);
            addChild(l);

            setTimeout(function() {
                trace(l.contentLoaderInfo.content);
            }, 100);
        }
    }
}
