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

            trace("multiple lines");
            var block:TextBlock = new TextBlock(
                new TextElement("ab\nc\td", new ElementFormat())
            );

            var line:TextLine = block.createTextLine(null, 10000);
            var index:int = 0;

            while (line) {
                trace("  line " + index + ": hasTabs=" + line.hasTabs);
                line = block.createTextLine(line, 10000);
                index++;
            }
        }

        private function testSingleLine(name:String, text:String):void {
            var block:TextBlock = new TextBlock(
                new TextElement(text, new ElementFormat())
            );

            var line:TextLine = block.createTextLine(null, 10000);

            trace(name + ": hasTabs=" + line.hasTabs);
        }
    }
}
