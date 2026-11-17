package {
    import flash.display.Sprite;
    import flash.text.engine.*;

    [SWF(width="650", height="260", frameRate="30")]
    public class Test extends Sprite {
        public function Test() {
            var samples:Array = [
                "ABC \u05E9\u05DC\u05D5\u05DD 123",
                "\u05E9\u05DC\u05D5\u05DD ABC 123"
            ];

            var font:FontDescription = new FontDescription();
            font.fontName = "DejaVu Sans";
            font.fontLookup = FontLookup.DEVICE;

            var format:ElementFormat = new ElementFormat(font, 28);

            for (var i:int = 0; i < samples.length; i++) {
                for (var level:int = 0; level <= 1; level++) {
                    var block:TextBlock = new TextBlock(
                        new TextElement(samples[i], format)
                    );

                    block.bidiLevel = level;

                    var line:TextLine = block.createTextLine(null, 300);

                    trace("CASE=" + i);
                    trace("LEVEL=" + level);
                    trace("LINE_NULL=" + (line == null));

                    if (line != null) {
                        line.x = 20 + level * 320;
                        line.y = 60 + i * 105;
                        addChild(line);

                        trace("RAW_LENGTH=" + line.rawTextLength);
                    }
                }
            }

            trace("DONE");
        }
    }
}
