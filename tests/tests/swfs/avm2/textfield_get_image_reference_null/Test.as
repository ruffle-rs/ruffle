package {
    import flash.display.DisplayObject;
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.utils.getQualifiedClassName;

    public class Test extends Sprite {
        public function Test() {
            probe("NULL", [null]);
            probe("UNDEFINED", [undefined]);
            probe("MISSING_ARGUMENT", []);
            probe("STRING_NULL", ["null"]);
            probe("STRING_UNDEFINED", ["undefined"]);
            probe("EMPTY_STRING", [""]);
        }

        private function probe(label:String, args:Array):void {
            trace("=== " + label + " ===");

            var tf:TextField = new TextField();
            tf.htmlText =
                "A<img id='null' src='test_image.jpg'>" +
                "B<img id='undefined' src='test_image.jpg'>C";

            var nullImage:DisplayObject = tf.getImageReference("null");
            var undefinedImage:DisplayObject =
                tf.getImageReference("undefined");

            trace("STRING_NULL_REF_EXISTS=" + (nullImage != null));
            trace("STRING_UNDEFINED_REF_EXISTS=" +
                (undefinedImage != null));

            var lookup:Function = tf.getImageReference;
            try {
                var result:DisplayObject = lookup.apply(tf, args);
                trace("THREW=false");
                trace("RESULT_NULL=" + (result == null));
                trace("RESULT_IS_NULL_IMAGE=" + (result === nullImage));
                trace("RESULT_IS_UNDEFINED_IMAGE=" +
                    (result === undefinedImage));
            } catch (error:Error) {
                trace("THREW=true");
                trace("ERROR_CLASS=" + getQualifiedClassName(error));
                trace("ERROR_ID=" + error.errorID);
            }
        }
    }
}
