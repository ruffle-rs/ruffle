// A root movie larger than Flash Player's 4 KiB read chunk: frame 1 runs
// before the rest of the file has arrived, which comes right after it, with
// a `progress` event per chunk. test.swf is this class on frame 1 plus filler
// frames (DefineBinaryData), uncompressed.
package {
    import flash.display.MovieClip;
    import flash.events.Event;
    import flash.events.ProgressEvent;
    import flash.text.TextField;

    public class Test extends MovieClip {
        private var tf:TextField = new TextField();
        private var ticks:int = 0;

        private function log(s:String):void {
            trace(s);
            tf.appendText(s + "\n");
        }

        private function state(what:String):void {
            log(what + ": currentFrame " + currentFrame + ", framesLoaded " + framesLoaded + "/" + totalFrames +
                ", bytes " + loaderInfo.bytesLoaded + "/" + loaderInfo.bytesTotal);
        }

        public function Test() {
            tf.width = 550;
            tf.height = 400;
            tf.multiline = true;
            addChild(tf);
            state("constructor");
            loaderInfo.addEventListener(ProgressEvent.PROGRESS, function(e:ProgressEvent):void {
                state("progress " + e.bytesLoaded + "/" + e.bytesTotal);
            });
            loaderInfo.addEventListener(Event.INIT, function(e:Event):void { state("init"); });
            loaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void { state("complete"); });
            addEventListener(Event.ENTER_FRAME, function(e:Event):void {
                ticks++;
                if (ticks <= 3) {
                    state("enterFrame " + ticks);
                }
            });
        }
    }
}
