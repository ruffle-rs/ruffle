package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.text.TextField;

    public class Test extends Sprite {
        private var tf:TextField;
        private var image:Loader;
        private var orphan:Loader;
        private var seenImage:Object = {};
        private var seenOrphan:Object = {};
        private var phase:int = 0;
        private var frames:int = 0;

        public function Test() {
            tf = new TextField();
            tf.width = 550;
            tf.height = 400;
            addChild(tf);
            tf.htmlText = "A<img id='probe' src='test_image.jpg'>B";

            image = tf.getImageReference("probe") as Loader;
            orphan = new Loader();

            watch(image, onImageFrame);
            watch(orphan, onOrphanFrame);

            image.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
            image.contentLoaderInfo.addEventListener(Event.UNLOAD, onUnload);
            addEventListener(Event.EXIT_FRAME, onRootFrame);
        }

        private function watch(loader:Loader, listener:Function):void {
            loader.addEventListener(Event.ENTER_FRAME, listener);
            loader.addEventListener(Event.FRAME_CONSTRUCTED, listener);
            loader.addEventListener(Event.EXIT_FRAME, listener);
        }

        private function onImageFrame(event:Event):void {
            seenImage[event.type] = true;
        }

        private function onOrphanFrame(event:Event):void {
            seenOrphan[event.type] = true;
        }

        private function onComplete(event:Event):void {
            trace("COMPLETE");
            trace("CONTENT_BEFORE_NULL=" + (image.content == null));
            trace("PARENT_BEFORE_NULL=" + (image.parent == null));
            trace("STAGE_BEFORE_NULL=" + (image.stage == null));

            seenImage = {};
            seenOrphan = {};
            frames = 0;
            phase = 1;
        }

        private function reportEvents(label:String):void {
            trace("=== " + label + " ===");
            var types:Array = [
                Event.ENTER_FRAME,
                Event.FRAME_CONSTRUCTED,
                Event.EXIT_FRAME
            ];
            for each (var type:String in types) {
                trace("IMAGE_" + type + "=" + (seenImage[type] === true));
                trace("ORPHAN_" + type + "=" + (seenOrphan[type] === true));
            }
        }

        private function onRootFrame(event:Event):void {
            if (phase == 0 || phase == 3) {
                return;
            }
            frames++;
            if (frames < 3) {
                return;
            }

            if (phase == 1) {
                reportEvents("BEFORE_UNLOAD");
                var textBefore:String = tf.text;
                var htmlBefore:String = tf.htmlText;

                trace("CALL_UNLOAD");
                image.unload();

                trace("CONTENT_AFTER_NULL=" + (image.content == null));
                trace("REFERENCE_AFTER_SAME=" +
                    (tf.getImageReference("probe") === image));
                trace("TEXT_AFTER_SAME=" + (tf.text == textBefore));
                trace("HTML_AFTER_SAME=" + (tf.htmlText == htmlBefore));
                trace("PARENT_AFTER_NULL=" + (image.parent == null));
                trace("WIDTH_AFTER=" + image.width);
                trace("HEIGHT_AFTER=" + image.height);

                seenImage = {};
                seenOrphan = {};
                frames = 0;
                phase = 2;
            } else {
                reportEvents("AFTER_UNLOAD");
                trace("LATER_CONTENT_NULL=" + (image.content == null));
                trace("LATER_REFERENCE_SAME=" +
                    (tf.getImageReference("probe") === image));
                phase = 3;

                unwatch(image, onImageFrame);
                unwatch(orphan, onOrphanFrame);
                removeEventListener(Event.EXIT_FRAME, onRootFrame);
            }
        }

        private function onUnload(event:Event):void {
            trace("UNLOAD_EVENT");
        }

        private function unwatch(loader:Loader, listener:Function):void {
            loader.removeEventListener(Event.ENTER_FRAME, listener);
            loader.removeEventListener(Event.FRAME_CONSTRUCTED, listener);
            loader.removeEventListener(Event.EXIT_FRAME, listener);
        }
    }
}
