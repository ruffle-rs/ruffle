package {
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.text.TextFormat;

    [SWF(width="440", height="50", frameRate="24")]
    public class Test extends Sprite {
        public function Test() {
            addSample("ABC\u202BDEF123\u202CXYZ", 10);
            trace("done");
        }

        private function addSample(text:String, y:Number):void {
            var field:TextField = new TextField();
            field.x = 20;
            field.y = y;
            field.width = 400;
            field.height = 40;
            field.selectable = false;
            field.defaultTextFormat =
                new TextFormat("DejaVu Sans", 22, 0x000000);
            field.text = text;
            addChild(field);
        }
    }
}
