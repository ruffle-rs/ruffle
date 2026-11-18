package {
    import flash.display.MovieClip;
    import flash.text.TextField;
    import flash.text.TextFieldType;
    import flash.events.Event;
    import flash.events.TextEvent;
    import flash.events.KeyboardEvent;

    public class Test extends MovieClip {
        private var singleLine:TextField;
        private var multiLine:TextField;
        private var currentField:int = 0;

        public function Test() {
            singleLine = createField("singleLine", 0, false);
            multiLine = createField("multiLine", 100, true);

            stage.addEventListener(KeyboardEvent.KEY_DOWN, function(e:KeyboardEvent):void {
                if (e.keyCode == 27) { // Escape key switches field / prints final
                    if (currentField == 0) {
                        trace("singleLine result: " + escapeNewlines(singleLine.text));
                        stage.focus = multiLine;
                        multiLine.setSelection(multiLine.text.length, multiLine.text.length);
                        currentField = 1;
                    } else {
                        trace("multiLine result: " + escapeNewlines(multiLine.text));
                    }
                }
            });

            stage.focus = singleLine;
            singleLine.setSelection(0, 0);
        }

        private function createField(name:String, y:Number, multiline:Boolean):TextField {
            var tf:TextField = new TextField();
            tf.type = TextFieldType.INPUT;
            tf.multiline = multiline;
            tf.width = 200;
            tf.height = 80;
            tf.y = y;
            tf.restrict = "A-Za-z0-9";
            tf.text = "";
            addChild(tf);

            tf.addEventListener(TextEvent.TEXT_INPUT, function(e:TextEvent):void {
                trace(name + " textInput: " + escapeNewlines(e.text));
            });
            tf.addEventListener(Event.CHANGE, function(e:Event):void {
                trace(name + " change: " + escapeNewlines(tf.text));
            });

            return tf;
        }

        private function escapeNewlines(str:String):String {
            if (str == null) return "null";
            return str.split("\r").join("\\r").split("\n").join("\\n");
        }
    }
}
