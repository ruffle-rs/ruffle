package {
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.text.TextField;
    import flash.utils.setTimeout;

    [SWF(width="640", height="480", frameRate="24")]
    public class Test extends Sprite {
        private var renderCount:int = 0;

        public function Test() {
            var background:Shape = new Shape();
            background.graphics.beginFill(0xEEEEEE);
            background.graphics.drawRect(0, 0, 640, 480);
            background.graphics.endFill();
            addChild(background);

            var immediate:TextField = makeField(true);
            var later:TextField = makeField(true);
            var offstage:TextField = makeField(false);

            stage.addEventListener(Event.ENTER_FRAME, function(event:Event):void {
                stage.invalidate();
            });
            stage.addEventListener(Event.RENDER, function(event:Event):void {
                if (++renderCount != 2) return;
                immediate.autoSize = "center";
                later.autoSize = "center";
                offstage.autoSize = "center";

                // A synchronous read observes the pending layout after cancelling it.
                immediate.wordWrap = true;
                trace("CASE synchronous width=" + immediate.width + " x=" + immediate.x);

                // Queue from RENDER so the callback follows this display refresh.
                setTimeout(function():void {
                    later.wordWrap = true;
                    offstage.wordWrap = true;
                    trace("CASE timeout width=" + later.width + " x=" + later.x);
                    trace("CASE offstage width=" + offstage.width + " x=" + offstage.x);
                }, 0);
            });
        }

        private function makeField(onstage:Boolean):TextField {
            var parent:Sprite = new Sprite();
            var text:TextField = new TextField();
            text.width = 200;
            text.visible = false;
            parent.addChild(text);
            if (onstage) addChild(parent);
            return text;
        }
    }
}
