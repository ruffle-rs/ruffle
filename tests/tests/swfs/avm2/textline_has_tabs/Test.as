package
{
    import flash.display.Sprite;
    import flash.text.engine.ElementFormat;
    import flash.text.engine.FontDescription;
    import flash.text.engine.FontLookup;
    import flash.text.engine.TextBlock;
    import flash.text.engine.TextElement;
    import flash.text.engine.TextLine;

    public class Test extends Sprite
    {
        [Embed(
            source="TextLineHasTabs.ttf",
            fontName="TextLineHasTabs",
            embedAsCFF="true",
            unicodeRange="U+0009-U+000B,U+001C-U+001F,U+0020,U+0044-U+0046,U+0061-U+0066,U+0088-U+008A,U+00A0,U+1680,U+180E,U+2000-U+200B,U+202F,U+205F,U+21B9,U+21E4-U+21E5,U+2409,U+240B,U+2B7E-U+2B7F,U+3000,U+FEFF"
        )]
        private var TestFont:Class;

        private var format:ElementFormat;

        public function Test()
        {
            var font:FontDescription = new FontDescription();
            font.fontName = "TextLineHasTabs";
            font.fontLookup = FontLookup.EMBEDDED_CFF;
            format = new ElementFormat(font);

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

            trace("individual code points");

            var codePoints:Array = [
                    ["0009", "\u0009"],
                    ["000B", "\u000B"],
                    ["001C", "\u001C"],
                    ["001D", "\u001D"],
                    ["001E", "\u001E"],
                    ["001F", "\u001F"],
                    ["0088", "\u0088"],
                    ["0089", "\u0089"],
                    ["008A", "\u008A"],
                    ["2409", "\u2409"],
                    ["240B", "\u240B"],
                    ["21B9", "\u21B9"],
                    ["21E5", "\u21E5"],
                    ["21E4", "\u21E4"],
                    ["2B7E", "\u2B7E"],
                    ["2B7F", "\u2B7F"],
                    ["0020", "\u0020"],
                    ["00A0", "\u00A0"],
                    ["1680", "\u1680"],
                    ["2000", "\u2000"],
                    ["2001", "\u2001"],
                    ["2002", "\u2002"],
                    ["2003", "\u2003"],
                    ["2004", "\u2004"],
                    ["2005", "\u2005"],
                    ["2006", "\u2006"],
                    ["2007", "\u2007"],
                    ["2008", "\u2008"],
                    ["2009", "\u2009"],
                    ["200A", "\u200A"],
                    ["202F", "\u202F"],
                    ["205F", "\u205F"],
                    ["3000", "\u3000"],
                    ["200B", "\u200B"],
                    ["180E", "\u180E"],
                    ["FEFF", "\uFEFF"]
                ];

            for each (var entry:Array in codePoints)
            {
                testCodePoint(entry[0], entry[1]);
            }
        }

        private function testCodePoint(hex:String, character:String):void
        {
            var text:String = "a" + character + "b";

            var block:TextBlock = new TextBlock(
                    new TextElement(text, format)
                );

            var line:TextLine = block.createTextLine(null, 10000);

            if (line)
            {
                trace("U+" + hex + ": hasTabs=" + line.hasTabs);
            }
            else
            {
                trace("U+" + hex + ": no line");
            }
        }

        private function testSingleLine(name:String, text:String):void
        {
            var block:TextBlock = new TextBlock(
                    new TextElement(text, format)
                );

            var line:TextLine = block.createTextLine(null, 10000);

            if (line)
            {
                trace(name + ": hasTabs=" + line.hasTabs);
            }
            else
            {
                trace(name + ": no line");
            }
        }

        private function testLines(name:String, text:String):void
        {
            trace(name);

            var block:TextBlock = new TextBlock(
                    new TextElement(text, format)
                );

            var line:TextLine = block.createTextLine(null, 10000);
            var index:int = 0;

            while (line)
            {
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
