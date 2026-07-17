package {
import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="true", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "TestFont";
        fd.fontLookup = FontLookup.EMBEDDED_CFF;

        var ef:ElementFormat = new ElementFormat(fd, 20);

        test(ef, "null", null);
        test(ef, "empty", "");
        test(ef, "space", " ");
        test(ef, "lf", "\n");
        test(ef, "cr", "\r");
        test(ef, "crlf", "\r\n");
        test(ef, "tab", "\t");
        test(ef, "spaces", "  ");
        test(ef, "whitespace2", " \n ");
        test(ef, "whitespace3", " \n\n ");
        test(ef, "whitespace4", " \n \n ");
        test(ef, "whitespace4", " \n  \n ");
        test(ef, "text1", "a");
        test(ef, "text2", "aa\naa");
        test(ef, "text3", "aa \naa");
        test(ef, "text4", "aa\n aa");
        test(ef, "text5", "aa \n aa");
        test(ef, "text6", "aa \na\n aa");
        test(ef, "text7", "a\ta\na\ta\n");
        test(ef, "text8", "a\n");
        test(ef, "text9", "\na");
    }

    private function test(ef:ElementFormat, name:String, text:String):void {
        var block:TextBlock = new TextBlock(new TextElement(text, ef));

        trace(name + ": " + block + " (" + block.textLineCreationResult + ")");
        var line:TextLine = block.createTextLine(null, 10000);
        while (line !== null) {
            if (line.textBlock !== block) {
                trace("WRONG BLOCK");
            }

            trace("  line: " + block.textLineCreationResult + ", " + line.textBlockBeginIndex + ", " + line.previousLine + ", " + line.nextLine + ", " + line.validity + ";");

            var nextLine:TextLine = block.createTextLine(line, 10000);
            if (nextLine !== null) {
                if (nextLine.previousLine !== line) {
                    trace("WRONG PREVIOUS LINE");
                }
                if (nextLine !== line.nextLine) {
                    trace("WRONG NEXT LINE");
                }
            }
            line = nextLine;
        }
        trace("  done: " + block.textLineCreationResult);
    }
}
}
