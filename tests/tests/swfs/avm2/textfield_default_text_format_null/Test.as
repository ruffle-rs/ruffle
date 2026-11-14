package {
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.text.TextFormat;

    public class Test extends Sprite {
        private var field:TextField = new TextField();

        public function Test() {
            trace("initial");
            print();

            var fmt:TextFormat = new TextFormat();
            fmt.size = 30;
            fmt.bold = true;
            fmt.color = 0xFF0000;
            field.defaultTextFormat = fmt;
            trace("after setting a format");
            print();

            trace("setting null");
            try {
                field.defaultTextFormat = null;
            } catch (e:Error) {
                trace("  " + Object(e).constructor);
                trace("  " + e.getStackTrace());
            }
            print();

            trace("setting null on a field with text");
            field.text = "AB";
            try {
                field.defaultTextFormat = null;
            } catch (e:Error) {
                trace("  " + Object(e).constructor);
                trace("  " + e.getStackTrace());
            }
            print();
        }

        private function print():void {
            var tf:TextFormat = field.defaultTextFormat;
            trace("  default: size=" + tf.size + " bold=" + tf.bold + " color=" + tf.color);
            field.text = "";
            field.text = "AB";
            tf = field.getTextFormat(0, 1);
            trace("  new text: size=" + tf.size + " bold=" + tf.bold + " color=" + tf.color);
        }
    }
}
