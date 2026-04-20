package {
    import flash.display.Sprite;
    import flash.display.Stage;
    import flash.events.Event;
    import flash.events.KeyboardEvent;
    import flash.events.TextEvent;
    import flash.ui.Keyboard;
    import flash.text.TextField;

    public class Test extends Sprite {
        public function Test() {
            var text = new TextField();
            text.border = true;
            text.width = 200;
            text.height = 20;
            text.type = "input";
            text.restrict = "x";
            text.maxChars = 5;
            addChild(text);

            stage.focus = text;
            stage.addEventListener("keyDown", function(event:KeyboardEvent):void {
                if (event.keyCode < 256) {
                    trace("key down: " + event.keyCode + ", " + text.text);
                }
            });
            stage.addEventListener("keyUp", function(event:KeyboardEvent):void {
                if (event.keyCode < 256) {
                    trace("key up: " + event.keyCode + ", " + text.text);
                }
            });
            text.addEventListener("textInput", function(event:TextEvent):void {
                trace("text input: " + event.text + ", " + text.text);
            });
            text.addEventListener("change", function(event:*):void {
                trace("changed: " + text.text);
            });
        }
    }
}
