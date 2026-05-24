package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="200", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        fd.fontLookup = FontLookup.DEVICE;
        var fmt:ElementFormat = new ElementFormat(fd, 50);
        var smallFmt:ElementFormat = new ElementFormat(fd, 25);
        var line:TextLine = new TextBlock(new TextElement("Hello", fmt))
            .createTextLine(null, 100000);
        var narrow:TextLine = new TextBlock(new TextElement("i", fmt))
            .createTextLine(null, 100000);
        var wide:TextLine = new TextBlock(new TextElement("W", fmt))
            .createTextLine(null, 100000);
        var small:TextLine = new TextBlock(new TextElement("Hello", smallFmt))
            .createTextLine(null, 100000);
        var wrappedText:String = "one two three";
        var narrowBlock:TextBlock = new TextBlock(new TextElement(wrappedText, new ElementFormat(fd, 30)));
        var narrowLine:TextLine = narrowBlock.createTextLine(null, 60);
        var maxLine:TextLine = new TextBlock(new TextElement(wrappedText, new ElementFormat(fd, 30)))
            .createTextLine(null, TextLine.MAX_LINE_WIDTH);
        var aboveMaxLine:TextLine = new TextBlock(new TextElement(wrappedText, new ElementFormat(fd, 30)))
            .createTextLine(null, TextLine.MAX_LINE_WIDTH + 1);

        trace("textWidth follows glyph advance (these are still stubs): " + (wide.textWidth > narrow.textWidth));
        trace("textHeight equals ascent plus descent (these are still stubs): " + near(line.textHeight, line.ascent + line.descent));
        trace("larger font has larger ascent (these are still stubs): " + (line.ascent > small.ascent));
        trace("larger font has larger descent (these are still stubs): " + (line.descent > small.descent));
        trace("larger font has larger textHeight (these are still stubs): " + (line.textHeight > small.textHeight));
        trace("ascent is not fixed fallback (these are still stubs): " + !near(line.ascent, 12));
        trace("descent is not fixed fallback (these are still stubs): " + !near(line.descent, 3));
        trace("max line width: " + TextLine.MAX_LINE_WIDTH);
        trace("narrow width wraps: " + (narrowLine.rawTextLength < wrappedText.length));
        trace("max width takes full text: " + (maxLine.rawTextLength == wrappedText.length));
        trace("above max takes full text: " + (aboveMaxLine.rawTextLength == wrappedText.length));
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.01;
    }
}
}
