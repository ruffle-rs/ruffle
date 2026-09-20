package {
    import flash.display.Sprite;
    import flash.text.engine.ElementFormat;
    import flash.text.engine.TextBlock;
    import flash.text.engine.TextElement;
    import flash.text.engine.TextLine;

    public class Test extends Sprite {
        public function Test() {
            testSingleLine("plain text", "abc");
            testSingleLine("text with tab", "a\tb");
            testSingleLine("tab only", "\t");
            testSingleLine("empty text", "");

            testLines(
                "tab in second line / first line ends before tab",
                "abc\n\tdef"
            );

            testLines(
                "tab in first line only",
                "abc\t\nDEF"
            );
        }

        private function testSingleLine(name:String, text:String):void {
            var block:TextBlock = new TextBlock(
                new TextElement(text, new ElementFormat())
            );

            var line:TextLine = block.createTextLine(null, 10000);

            if (line) {
                trace(name + ": hasTabs=" + line.hasTabs);
            } else {
                trace(name + ": no line");
            }
        }

        private function testLines(name:String, text:String):void {
            trace(name);

            var block:TextBlock = new TextBlock(
                new TextElement(text, new ElementFormat())
            );

            var line:TextLine = block.createTextLine(null, 10000);
            var index:int = 0;

            while (line) {
                trace(
                    "  line " + index +
                    ": begin=" + line.textBlockBeginIndex +
                    ", length=" + line.rawTextLength +
                    ", hasTabs=" + line.hasTabs
                );

                line = block.createTextLine(line, 10000);
                index++;
            }
        }
    }
}