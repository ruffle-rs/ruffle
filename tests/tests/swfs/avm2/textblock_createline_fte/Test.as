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

        var elements:Vector.<ContentElement> = new Vector.<ContentElement>();
        elements.push(new TextElement("Hello ", fmt));
        elements.push(new TextElement("World", fmt));
        var group:GroupElement = new GroupElement(elements, fmt);
        var tb3:TextBlock = new TextBlock(group);
        var line3:TextLine = tb3.createTextLine(null, 500);
        trace("line3.textBlockBeginIndex: " + line3.textBlockBeginIndex)
        trace("line3.rawTextLength: " + line3.rawTextLength)
        trace("tb3.textLineCreationResult: " + tb3.textLineCreationResult);
        // Now shrink the contents, so that current index + length > new length.
        group.replaceElements(1, 2, null);
        trace("createTextLine: " + tb3.createTextLine(line3, 500));
        trace("tb3.textLineCreationResult: " + tb3.textLineCreationResult);
    }
}
}
