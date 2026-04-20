package  {
    import flash.text.TextField;
    import flash.text.TextFormat;
    import flash.display.MovieClip;
    import flash.display.StageQuality;
    import flash.ui.Keyboard;
    import flash.events.Event;
    import flash.events.KeyboardEvent;
    import flash.events.MouseEvent;

    [SWF(width="400", height="400")]
    public class Test extends MovieClip {
        private var text:TextField;

        public function Test() {
            stage.quality = StageQuality.HIGH;
            stage.addEventListener(KeyboardEvent.KEY_DOWN, onKeyDown);
            stage.addEventListener(MouseEvent.MOUSE_DOWN, onMouseDown);
            stage.addEventListener(MouseEvent.MOUSE_UP, onMouseUp);

            text = new TextField();
            text.x = 0;
            text.y = 100;
            text.width = 400;
            text.height = 100;
            text.addEventListener(Event.CHANGE, onTextChange);
            text.border = true;
            text.type = "input";

            var tf:TextFormat = new TextFormat();
            tf.size = 30;
            text.defaultTextFormat = tf;
            addChild(text);

            trace("Loaded!");
        }

        private function onKeyDown(event: KeyboardEvent): void {
            if (event.keyCode == Keyboard.Q) {
                trace("quality: " + stage.quality);
                stage.quality = StageQuality.HIGH;
            }
            if (event.keyCode == Keyboard.T) {
                trace("populating text");
                text.text = "example";
                stage.focus = text;
            }
        }

        private function onMouseDown(event: MouseEvent): void {
            trace("onMouseDown()");
        }

        private function onMouseUp(event: MouseEvent): void {
            trace("onMouseUp()");
        }

        private function onTextChange(event: Event): void {
            trace("text changed: " + text.text);
        }
    }
}
