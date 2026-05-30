package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="100", height="100")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="true", unicodeRange="U+0020-U+007E")]
    private var TestFont:Class;

    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "TestFont";
        fd.fontLookup = FontLookup.EMBEDDED_CFF;
        var fmt:ElementFormat = new ElementFormat(fd, 14);

        var tb:TextBlock = new TextBlock(new TextElement("Hello", fmt));
        var line:TextLine = tb.createTextLine(null, 500);
        trace("line != null: " + (line != null));
        trace("result: " + tb.textLineCreationResult);
        trace("firstLine === line: " + (tb.firstLine === line));

        var tb2:TextBlock = new TextBlock();
        trace("null content: " + tb2.createTextLine(null, 500));
    }
}
}
