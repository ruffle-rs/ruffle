package {
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.events.FocusEvent;

    public class Test extends Sprite {
        public function Test() {
            var textField1 = new TextField();
            textField1.text = "Field 1";
            textField1.border = true;
            textField1.addEventListener(FocusEvent.FOCUS_IN, onFocus);
            addChild(textField1);
            var textField2 = new TextField();
            textField2.text = "Field 2";
            textField2.border = true;
            textField2.x = 200;
            textField2.addEventListener(FocusEvent.FOCUS_IN, onFocus);
            addChild(textField2);
        }

        function onFocus(evt: FocusEvent) {
            trace("evt.type: " + evt.type);
            trace("evt.target: " + evt.target);
            trace("evt.target.text: " + evt.target.text);
            trace("evt.relatedObject: " + evt.relatedObject);
            if (evt.relatedObject) {
                var related = evt.relatedObject as TextField;
                trace("related.text: " + related.text);
            }
        }
    }
}
