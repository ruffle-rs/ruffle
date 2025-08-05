package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.KeyboardEvent;
    import flash.events.TextEvent;
    import flash.text.TextField;
    import flash.text.TextFieldType;

    public class Test extends Sprite {
        public function Test() {
            var textField: TextField = new TextField();
            textField.type = TextFieldType.INPUT;
            textField.border = true;
            textField.multiline = true;
            textField.height = 200;
            textField.addEventListener(Event.CHANGE, onChange);
            textField.addEventListener(KeyboardEvent.KEY_DOWN, onKeyDown);
            textField.addEventListener(TextEvent.TEXT_INPUT, onTextInput);
            addChild(textField);
        }

        public function onChange(evt: Event): void {
            trace("onChange");

            var text: String = evt.target.text;
            for (var i: int = 0; i < text.length; i++) {
                trace(" U+" + text.charCodeAt(i).toString(16));
            }
        }

        public function onKeyDown(evt: KeyboardEvent): void {
            trace("onKeyDown: U+" + evt.charCode.toString(16));
        }

        public function onTextInput(evt: TextEvent): void {
            trace("onTextInput: U+" + evt.text.charCodeAt(0).toString(16));
        }
    }
}
