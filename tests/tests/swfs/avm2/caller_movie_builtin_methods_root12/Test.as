package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;

    // The checks run from this movie, then from loaded movies with lower and
    // higher SWF versions.
    public class Test extends Sprite {
        private var children:Array = ["child10.swf", "child12.swf", "child19.swf"];

        public function Test() {
            trace("/// test.swf");
            Checks.run();
            loadNext();
        }

        private function loadNext():void {
            if (children.length == 0) {
                return;
            }
            var url:String = children.shift();
            trace("/// " + url);
            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                loadNext();
            });
            loader.load(new URLRequest(url));
        }
    }
}
