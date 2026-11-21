package {
    import flash.display.Sprite;
    import flash.events.TextEvent;
    import flash.text.TextField;

    public class Test extends Sprite {
        public function Test() {
            var tf:TextField = new TextField();
            tf.width = 400;
            tf.height = 80;
            tf.htmlText = '<font size="32"><a href="EvEnT:greetings">CLICKME</a></font>';
            tf.addEventListener(TextEvent.LINK, function(e:TextEvent):void {
                trace("called: " + e.text);
            });
            addChild(tf);
        }
    }
}
