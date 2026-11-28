package {
    import flash.display.Sprite;
    import flash.geom.Rectangle;
    import flash.text.TextField;
    import flash.text.TextFormat;

    [SWF(width="620", height="500", frameRate="24")]
    public class Test extends Sprite {
        public function Test() {
            var samples:Array = [
                "\u05E9\u05DC\u05D5\u05DD",
                "\u0645\u0631\u062D\u0628\u0627",
                "\u0633\u0644\u0627\u0645 \u06F1\u06F2\u06F3 ABC",
                "ABC (\u05E9\u05DC\u05D5\u05DD) 123",
                "\u05E9\u05DC\u05D5\u05DD (ABC) 123",
                "\u200F\u05E9\u05DC\u05D5\u05DD ABC 123",
                "\u200E\u05E9\u05DC\u05D5\u05DD ABC 123",
                "ABC \u202B\u05E9\u05DC\u05D5\u05DD 123\u202C XYZ",
                "ABC \u202EDEF 123\u202C XYZ",
                "\u202A\u05E9\u05DC\u05D5\u05DD ABC\u202C"
            ];

            for (var i:int = 0; i < samples.length; i++) {
                var field:TextField = new TextField();
                field.x = 20;
                field.y = 20 + i * 45;
                field.width = 550;
                field.height = 40;
                field.border = true;
                field.selectable = false;
                field.defaultTextFormat =
                    new TextFormat("DejaVu Sans", 22, 0x000000);
                field.text = samples[i];
                addChild(field);

                trace("sample " + i + " length=" + field.length);
                for (var j:int = 0; j < field.length; j++) {
                    var bounds:Rectangle =
                        field.getCharBoundaries(j);
                    trace("  index=" + j +
                        " code=" + field.text.charCodeAt(j) +
                        " hasBounds=" + (bounds != null));
                    if (j > 0 && bounds != null) {
                        var previous:Rectangle =
                            field.getCharBoundaries(j - 1);
                        if (previous != null) {
                            trace("    relativeX=" +
                                (bounds.x > previous.x ? "right" :
                                 bounds.x < previous.x ? "left" : "same"));
                        }
                    }
                    if (bounds != null && bounds.width > 0 &&
                        bounds.height > 0) {
                        trace("    hit=" + field.getCharIndexAtPoint(
                            bounds.x + bounds.width / 2,
                            bounds.y + bounds.height / 2));
                    }
                }
            }
        }
    }
}
