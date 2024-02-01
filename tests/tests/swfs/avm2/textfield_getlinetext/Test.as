package {
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.events.FocusEvent;

    public class Test extends Sprite {
        public function Test() {
            trace("///");
            var textField1 = new TextField();
            textField1.text = "hello world";
            trace(textField1.getLineText(0));
            try {
                trace(textField1.getLineText(1));
            } catch (e) {
                trace(e);
            }

            trace("///");
            var textField2 = new TextField();
            textField2.multiline = true;
            textField2.text = "line 1\nline 2";
            trace(textField2.getLineText(0));
            trace(textField2.getLineText(1));

            trace("///");
            var textField3 = new TextField();
            textField3.htmlText = "a <b>b</b> <i>i</i>";
            trace(textField3.getLineText(0));
        }
    }
}
