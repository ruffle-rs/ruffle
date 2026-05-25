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

var ALPHABET:String = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

var tb:TextBlock = makeBlock(ALPHABET, 12);
var tl:TextLine = tb.createTextLine(null, 1000);

trace("ascent > 0: " + (tl.ascent > 0));
trace("descent > 0: " + (tl.descent > 0));
trace("lineHeight > 0: " + (tl.ascent + tl.descent > 0));
trace("ascent > descent: " + (tl.ascent > tl.descent));

var tb2:TextBlock = makeBlock(ALPHABET, 24);
var tl2:TextLine = tb2.createTextLine(null, 2000);

trace("24px ascent > 12px ascent: " + (tl2.ascent > tl.ascent));
trace("24px descent > 12px descent: " + (tl2.descent > tl.descent));

trace("ascent stable: " + (tl.ascent == tl.ascent));
trace("descent stable: " + (tl.descent == tl.descent));

function r(n:Number):Number {
    return Math.round(n * 10) / 10;
}

trace("12px ascent: " + r(tl.ascent));
trace("12px descent: " + r(tl.descent));
trace("24px ascent: " + r(tl2.ascent));
trace("24px descent: " + r(tl2.descent));
