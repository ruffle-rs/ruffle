package {
    import flash.display.Sprite;
    import flash.events.KeyboardEvent;
    import flash.text.TextField;

    public class Test extends Sprite {
        private var text:TextField;

        public function Test() {
            text = new TextField();
            text.border = true;
            text.width = 200;
            text.height = 40;
            text.type = "input";
            text.multiline = true;
            addChild(text);

            stage.focus = text;
            stage.addEventListener(KeyboardEvent.KEY_DOWN, keyPressedDown);
        }

        private function keyPressedDown(event:KeyboardEvent):void {
            if (event.keyCode == 27) {
                trace("Selected: " + text.selectedText.replace("\r", "\n"));
            }
        }
    }
}
