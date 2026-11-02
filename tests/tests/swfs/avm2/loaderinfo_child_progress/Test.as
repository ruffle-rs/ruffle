package {
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.events.Event;
    import flash.events.ProgressEvent;
    import flash.net.URLRequest;
    import flash.text.TextField;

    // The root movie's final 'progress' event comes after its frame 1 (see
    // loaderinfo_root_progress). A loaded movie gets its 'progress' events
    // before its constructor runs, so listeners added there see none.
    public class Test extends MovieClip {
        private static var tf:TextField = new TextField();
        private var loader:Loader = new Loader();

        public static function log(s:String):void {
            trace(s);
            tf.appendText(s + "\n");
        }

        public function Test() {
            tf.width = 550;
            tf.height = 400;
            addChild(tf);
            loader.contentLoaderInfo.addEventListener(ProgressEvent.PROGRESS, function(e:ProgressEvent):void {
                log("loader progress: " + e.bytesLoaded + " / " + e.bytesTotal);
            });
            loader.contentLoaderInfo.addEventListener(Event.INIT, function(e:Event):void {
                log("loader init");
            });
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                log("loader complete");
            });
            addChild(loader);
            loader.load(new URLRequest("child.swf"));
        }
    }
}
