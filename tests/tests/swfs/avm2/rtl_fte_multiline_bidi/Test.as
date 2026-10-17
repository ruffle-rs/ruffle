package {
    import flash.display.Sprite;
    import flash.text.engine.*;

    [SWF(width="650", height="260", frameRate="30")]
    public class Test extends Sprite {
        public function Test() {
            var font:FontDescription = new FontDescription();
            font.fontName = "DejaVu Sans";
            font.fontLookup = FontLookup.DEVICE;

            var format:ElementFormat = new ElementFormat(font, 28);

            var text:String =
                "ABC \u05E9\u05DC\u05D5\u05DD 123\n" +
                "\u05E9\u05DC\u05D5\u05DD ABC 123";

            for (var level:int = 0; level <= 1; level++) {
                var block:TextBlock = new TextBlock(
                    new TextElement(text, format)
                );

                block.bidiLevel = level;

                var previous:TextLine = null;

                for (var i:int = 0; i < 2; i++) {
                    var line:TextLine =
                        block.createTextLine(previous, 300);

                    trace("LEVEL=" + level);
                    trace("LINE=" + i);
                    trace("LINE_NULL=" + (line == null));

                    if (line == null) {
                        break;
                    }

                    trace("RAW_LENGTH=" + line.rawTextLength);

                    line.x = 20 + level * 320;
                    line.y = 65 + i * 90;
                    addChild(line);

                    previous = line;
                }
            }

            trace("DONE");
        }
    }
}
