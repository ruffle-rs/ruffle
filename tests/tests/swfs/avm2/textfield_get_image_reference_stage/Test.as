package {
    import flash.display.DisplayObject;
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.text.TextField;

    public class Test extends Sprite {
        private var tf:TextField;
        private var loader:Loader;
        private var savedContent:DisplayObject;
        private var frames:int = 0;

        public function Test() {
            tf = new TextField();
            tf.width = 550;
            tf.height = 400;
            tf.htmlText = "A<img id='probe' src='test_image.jpg'>B";
            loader = tf.getImageReference("probe") as Loader;

            snapshot("INITIAL_DETACHED");
            addChild(tf);
            snapshot("INITIAL_ATTACHED");
            removeChild(tf);
            snapshot("INITIAL_REMOVED");

            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
        }

        private function snapshot(label:String):void {
            trace("=== " + label + " ===");
            trace("TF_STAGE_NULL=" + (tf.stage == null));
            trace("LOADER_STAGE_NULL=" + (loader.stage == null));
            trace("LOADER_STAGE_IS_ROOT_STAGE=" + (loader.stage === stage));
            trace("LOADER_PARENT_NULL=" + (loader.parent == null));
            trace("CONTENT_NULL=" + (loader.content == null));
            trace("NUM_CHILDREN=" + loader.numChildren);
            trace("WIDTH=" + loader.width);
            trace("HEIGHT=" + loader.height);
            trace("REFERENCE_SAME=" +
                (tf.getImageReference("probe") === loader));

            if (savedContent != null) {
                trace("SAVED_PARENT_IS_LOADER=" +
                    (savedContent.parent === loader));
                trace("SAVED_STAGE_NULL=" + (savedContent.stage == null));
                if (loader.numChildren > 0) {
                    trace("CHILD_IS_SAVED=" +
                        (loader.getChildAt(0) === savedContent));
                }
            }
        }

        private function onComplete(event:Event):void {
            savedContent = loader.content;
            snapshot("COMPLETE_DETACHED");

            addChild(tf);
            snapshot("COMPLETE_ATTACHED");
            removeChild(tf);
            snapshot("COMPLETE_REMOVED");

            addChild(tf);
            addEventListener(Event.EXIT_FRAME, onDelayedUnload);
        }

        private function onDelayedUnload(event:Event):void {
            frames++;
            if (frames < 3) {
                return;
            }
            removeEventListener(Event.EXIT_FRAME, onDelayedUnload);
            snapshot("BEFORE_DELAYED_UNLOAD");
            loader.unload();
            snapshot("UNLOADED_ATTACHED");
            removeChild(tf);
            snapshot("UNLOADED_REMOVED");
            addChild(tf);
            snapshot("UNLOADED_REATTACHED");
        }
    }
}
