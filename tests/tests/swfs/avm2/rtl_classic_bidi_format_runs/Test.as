package {
    import flash.display.Sprite;
    import flash.text.TextField;
    import flash.text.TextFormat;

    [SWF(width="620", height="310", frameRate="24")]
    public class Test extends Sprite {
        public function Test() {
            var samples:Array = [
                "\u05E9\u05DC\u05D5\u05DD",
                "ABC \u05E9\u05DC\u05D5\u05DD 123",
                "ABC \u05E9\u05DC\u05D5\u05DD 123"
            ];

            var starts:Array = [1, 5, 4];
            var ends:Array = [3, 7, 8];

            for (var i:int = 0; i < samples.length; i++) {
                for (var variant:int = 0; variant < 2; variant++) {
                    var row:int = i * 2 + variant;
                    var field:TextField = new TextField();
                    field.x = 20;
                    field.y = 15 + row * 48;
                    field.width = 550;
                    field.height = 40;
                    field.border = true;
                    field.selectable = false;
                    field.defaultTextFormat =
                        new TextFormat("DejaVu Sans", 22, 0x000000);
                    field.text = samples[i];

                    if (variant == 1) {
                        var color:TextFormat = new TextFormat();
                        color.color = 0xCC0000;
                        field.setTextFormat(color, starts[i], ends[i]);
                    }

                    addChild(field);
                    trace("row=" + row + " colored=" + (variant == 1));
                }
            }
        }
    }
}
