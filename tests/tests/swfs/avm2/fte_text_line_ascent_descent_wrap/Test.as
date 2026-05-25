package {
    import flash.display.Sprite;
    public class Test extends Sprite {}
}

import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.FontLookup;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

function makeBlock(text:String, size:Number):TextBlock {
    var font:FontDescription = new FontDescription();
    font.fontName = "Noto Sans";
    font.fontLookup = FontLookup.DEVICE;
    var fmt:ElementFormat = new ElementFormat(font);
    fmt.fontSize = size;
    var block:TextBlock = new TextBlock();
    block.content = new TextElement(text, fmt);
    return block;
}

function r(n:Number):Number {
    return Math.round(n * 10) / 10;
}

var block:TextBlock = makeBlock("The quick brown fox jumps over the lazy dog", 12);

var line1:TextLine = block.createTextLine(null, 60);
trace("line1 != null: " + (line1 != null));
if (line1 != null) {
    trace("line1 ascent: " + r(line1.ascent));
}

var line2:TextLine = block.createTextLine(line1, 60);
trace("line2 != null: " + (line2 != null));
if (line2 != null) {
    trace("line2 ascent: " + r(line2.ascent));
    trace("line2 ascent == line1 ascent: " + (line2.ascent == line1.ascent));
}
