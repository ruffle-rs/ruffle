package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="200", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
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

        trace("line.textWidth: " + rounded(line.textWidth));
        trace("narrow.textWidth: " + rounded(narrow.textWidth));
        trace("wide.textWidth: " + rounded(wide.textWidth));
        trace("line.textHeight: " + rounded(line.textHeight));
        trace("line.ascent: " + rounded(line.ascent));
        trace("line.descent: " + rounded(line.descent));
        trace("small.textHeight: " + rounded(small.textHeight));
        trace("small.ascent: " + rounded(small.ascent));
        trace("small.descent: " + rounded(small.descent));
        trace("max line width: " + TextLine.MAX_LINE_WIDTH);
        trace("narrowLine.rawTextLength: " + narrowLine.rawTextLength);
        trace("maxLine.rawTextLength: " + maxLine.rawTextLength);
        trace("aboveMaxLine.rawTextLength: " + aboveMaxLine.rawTextLength);
    }

    private function rounded(value:Number):Number {
        return Math.round(value * 100) / 100;
    }
}
}
