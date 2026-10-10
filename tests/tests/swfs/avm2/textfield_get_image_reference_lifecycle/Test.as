package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.text.TextField;

    public class Test extends Sprite {
        public function Test() {
            probe("CLEAR_TEXT");
            probe("REASSIGN_HTML");
            probe("REPLACE_IMAGE");
            probe("MOVE_LOADER");
        }

        private function probe(label:String):void {
            trace("=== " + label + " ===");
            var tf:TextField = new TextField();
            addChild(tf);
            var html:String = "A<img id='probe' src='test_image.jpg'>B";
            tf.htmlText = html;
            var old:Loader = tf.getImageReference("probe") as Loader;
            trace("BEFORE_STAGE_SAME=" + (old.stage === stage));
            trace("BEFORE_PARENT_NULL=" + (old.parent == null));

            var holder:Sprite;
            switch (label) {
                case "CLEAR_TEXT":
                    tf.text = "plain";
                    break;
                case "REASSIGN_HTML":
                    tf.htmlText = html;
                    break;
                case "REPLACE_IMAGE":
                    tf.replaceText(1, 2, "");
                    break;
                case "MOVE_LOADER":
                    holder = new Sprite();
                    addChild(holder);
                    try {
                        holder.addChild(old);
                        trace("MOVE_OK=true");
                        trace("MOVED_PARENT_SAME=" + (old.parent === holder));
                        trace("MOVED_STAGE_SAME=" + (old.stage === stage));
                        trace("MOVED_REFERENCE_SAME=" + (tf.getImageReference("probe") === old));
                        holder.removeChild(old);
                        trace("REMOVE_OK=true");
                    } catch (error:Error) {
                        trace("MOVE_OR_REMOVE_ERROR=" + error.errorID);
                    }
                    break;
            }

            trace("AFTER_PARENT_NULL=" + (old.parent == null));
            trace("AFTER_STAGE_NULL=" + (old.stage == null));
            trace("AFTER_STAGE_SAME=" + (old.stage === stage));
            trace("LOOKUP_NULL=" + (tf.getImageReference("probe") == null));
            trace("LOOKUP_OLD_SAME=" + (tf.getImageReference("probe") === old));
            removeChild(tf);
            trace("TF_REMOVED_OLD_STAGE_NULL=" + (old.stage == null));
            if (holder != null) {
                removeChild(holder);
            }
        }
    }
}
