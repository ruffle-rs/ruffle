package {
    import flash.display.DisplayObject;
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.text.TextFormat;
    import flash.utils.getQualifiedClassName;

    public class Test extends Sprite {
        public function Test() {
            var identical:String =
                "A<img id='dup' src='test_image.jpg'>" +
                "<img id='dup' src='test_image.jpg'>B";

            var different:String =
                "A<img id='dup' src='test_image.jpg' hspace='1'>" +
                "<img id='dup' src='test_image.jpg' hspace='2'>B";

            probe("IDENTICAL_REMOVE_FIRST", identical, 1, 2);
            probe("IDENTICAL_REMOVE_SECOND", identical, 2, 3);
            probe("IDENTICAL_REMOVE_BOTH", identical, 1, 3);
            probe("CONTROL_REMOVE_FIRST", different, 1, 2);
            probe("CONTROL_REMOVE_SECOND", different, 2, 3);
            uniqueProbe("UNIQUE_REMOVE_FIRST", 1, 2);
            uniqueProbe("UNIQUE_REMOVE_SECOND", 2, 3);
            renameProbe("IDENTICAL_RENAME", identical);
            renameProbe("CONTROL_RENAME", different);
            countProbe("COUNT_IDENTICAL_PAIR", identical, false);
            countProbe("COUNT_CONTROL_PAIR", different, false);
            countProbe("COUNT_SINGLE_WITH_SPACES",
                "A<img id='dup' src='test_image.jpg'>  B", false);
            countProbe("COUNT_IDENTICAL_TRIPLE",
                "A<img id='dup' src='test_image.jpg'>" +
                "<img id='dup' src='test_image.jpg'>" +
                "<img id='dup' src='test_image.jpg'>B", false);
            countProbe("COUNT_READBACK_REASSIGN", identical, true);

            uniqueProbe("UNIQUE_DELETE_PREFIX", 0, 1);
            uniqueProbe("UNIQUE_REPLACE_FIRST_WITH_X", 1, 2, "X");
            uniqueProbe("UNIQUE_INSERT_BETWEEN", 2, 2, "X");
            uniqueProbe("UNIQUE_REPLACE_PREFIX_LONG", 0, 1, "XYZ");
            uniqueProbe("UNIQUE_APPEND", 4, 4, "X");
            uniqueProbe("UNIQUE_REPLACE_SUFFIX", 3, 4, "X");



        }


        private function uniqueProbe(label:String, from:int, to:int, replacement:String = ""):void {
            trace("=== " + label + " ===");
            var tf:TextField = new TextField();
            tf.defaultTextFormat = new TextFormat("Arial", 12);
            tf.htmlText =
                "A<img id='first' src='test_image.jpg'>" +
                "<img id='second' src='test_image.jpg'>B";

            var first:DisplayObject = tf.getImageReference("first");
            var second:DisplayObject = tf.getImageReference("second");
            trace("FIRST_BEFORE_NULL=" + (first == null));
            trace("SECOND_BEFORE_NULL=" + (second == null));
            trace("BEFORE_DIFFERENT=" + (first !== second));

            tf.replaceText(from, to, replacement);

            var firstAfter:DisplayObject = tf.getImageReference("first");
            var secondAfter:DisplayObject = tf.getImageReference("second");
            trace("TEXT_AFTER=[" + tf.text + "]");
            trace("FIRST_AFTER_NULL=" + (firstAfter == null));
            trace("SECOND_AFTER_NULL=" + (secondAfter == null));
            trace("FIRST_SAME=" + (first === firstAfter));
            trace("SECOND_SAME=" + (second === secondAfter));
        }

        private function renameProbe(label:String, html:String):void {
            trace("=== " + label + " ===");
            var tf:TextField = new TextField();
            tf.defaultTextFormat = new TextFormat("Arial", 12);
            tf.htmlText = html;

            var before:DisplayObject = tf.getImageReference("dup");
            trace("BEFORE_NULL=" + (before == null));
            trace("LOOKUP_STABLE=" +
                (before === tf.getImageReference("dup")));
            if (before == null) {
                return;
            }

            before.name = "renamed";
            var dup:DisplayObject = tf.getImageReference("dup");
            var renamed:DisplayObject = tf.getImageReference("renamed");
            trace("DUP_AFTER_RENAME_NULL=" + (dup == null));
            trace("DUP_IS_BEFORE=" + (dup === before));
            trace("RENAMED_NULL=" + (renamed == null));
            trace("RENAMED_IS_BEFORE=" + (renamed === before));
            trace("TWO_DISTINCT_REFS=" +
                (dup != null && renamed != null && dup !== renamed));
            trace("HTML_AFTER_RENAME=[" + tf.htmlText + "]");

            before.name = "dup";
            trace("RESTORED_LOOKUP_SAME=" +
                (before === tf.getImageReference("dup")));
        }


        private function countProbe(
            label:String, html:String, roundtrip:Boolean
        ):void {
            trace("=== " + label + " ===");
            var tf:TextField = new TextField();
            tf.defaultTextFormat = new TextFormat("Arial", 12);
            tf.htmlText = html;
            if (roundtrip) {
                tf.htmlText = tf.htmlText;
            }
            trace("TEXT=[" + tf.text + "]");
            trace("HTML=[" + tf.htmlText + "]");

            var found:Array = [];
            for (var i:int = 0; i < 8; i++) {
                var ref:DisplayObject = tf.getImageReference("dup");
                if (ref == null) {
                    break;
                }
                if (found.indexOf(ref) >= 0) {
                    trace("REPEATED_OBJECT=true");
                    break;
                }
                found.push(ref);
                ref.name = "counted_" + i;
                trace("RENAMED_LOOKUP_SAME=" +
                    (tf.getImageReference("counted_" + i) === ref));
            }
            trace("LOADER_COUNT=" + found.length);
            trace("DUP_REMAINING_NULL=" +
                (tf.getImageReference("dup") == null));
        }

        private function probe(
            label:String, html:String, from:int, to:int
        ):void {
            trace("=== " + label + " ===");

            var tf:TextField = new TextField();
            tf.defaultTextFormat = new TextFormat("Arial", 12);
            tf.htmlText = html;

            var before:DisplayObject = tf.getImageReference("dup");

            trace("TEXT_BEFORE=[" + tf.text + "]");
            trace("LENGTH_BEFORE=" + tf.length);
            trace("HTML_BEFORE=[" + tf.htmlText + "]");
            trace("BEFORE_NULL=" + (before == null));
            if (before != null) {
                trace("BEFORE_CLASS=" + getQualifiedClassName(before));
            }

            tf.replaceText(from, to, "");

            var after:DisplayObject = tf.getImageReference("dup");

            trace("TEXT_AFTER=[" + tf.text + "]");
            trace("LENGTH_AFTER=" + tf.length);
            trace("HTML_AFTER=[" + tf.htmlText + "]");
            trace("AFTER_NULL=" + (after == null));
            trace("SAME_REFERENCE=" + (before === after));
        }
    }
}
