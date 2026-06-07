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

        trace("max line width: " + TextLine.MAX_LINE_WIDTH);
        traceLine("line", line);
        traceLine("narrow", narrow);
        traceLine("wide", wide);
        traceLine("small", small);
        traceLine("narrowLine", narrowLine);
        traceLine("maxLine", maxLine);
        traceLine("aboveMaxLine", aboveMaxLine);
    }

    private function traceLine(name:String, line:TextLine):void {
        trace(name + ".width: " + rounded(line.width));
        trace(name + ".textWidth: " + rounded(line.textWidth));
        trace(name + ".textHeight: " + rounded(line.textHeight));
        trace(name + ".ascent: " + rounded(line.ascent));
        trace(name + ".descent: " + rounded(line.descent));
        trace(name + ".rawTextLength: " + line.rawTextLength);
    }

    private function rounded(value:Number):Number {
        return Math.round(value * 100) / 100;
    }
}
}
