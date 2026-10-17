package {
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.text.TextFormat;

    [SWF(width="620", height="180", frameRate="24")]
    public class Test extends Sprite {
        public function Test() {
            for (var i:int = 0; i < 3; i++) {
                var field:TextField = new TextField();
                field.x = 20;
                field.y = 20 + i * 45;
                field.width = 550;
                field.height = 40;
                field.border = true;
                field.selectable = true;
                field.defaultTextFormat =
                    new TextFormat("DejaVu Sans", 22, 0x000000);
                field.text = "\u05E9\u05DC\u05D5\u05DD";
                field.alwaysShowSelection = true;
                addChild(field);

                if (i > 0) {
                    field.setSelection(0, i);
                }

                trace("row=" + i +
                    " selection=" + field.selectionBeginIndex +
                    "," + field.selectionEndIndex);
            }
        }
    }
}
