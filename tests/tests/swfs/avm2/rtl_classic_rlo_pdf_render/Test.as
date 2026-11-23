package {
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.text.TextFormat;

    [SWF(width="440", height="120", frameRate="24")]
    public class Test extends Sprite {
        public function Test() {
            addSample("ABCDEF 123XYZ", 20);
            addSample("ABC\u202EDEF 123\u202CXYZ", 65);
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
