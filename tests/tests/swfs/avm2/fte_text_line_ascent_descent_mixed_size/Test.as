package {
    import flash.display.Sprite;
    public class Test extends Sprite {}
}

import flash.text.engine.ContentElement;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.FontLookup;
import flash.text.engine.GroupElement;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

function fmt(size:Number):ElementFormat {
    var font:FontDescription = new FontDescription();
    font.fontName = "Noto Sans";
    font.fontLookup = FontLookup.DEVICE;
    var f:ElementFormat = new ElementFormat(font);
    f.fontSize = size;
    return f;
}

function makeLine(content:ContentElement):TextLine {
    var block:TextBlock = new TextBlock();
    block.content = content;
    return block.createTextLine(null, 2000);
}

var small:TextElement = new TextElement("small ", fmt(12));
var big:TextElement = new TextElement("BIG", fmt(48));
var mixed:TextLine = makeLine(new GroupElement(new <ContentElement>[small, big]));

var ref:TextLine = makeLine(new TextElement("BIG", fmt(48)));

trace("mixed ascent == 48px ascent: " + (mixed.ascent == ref.ascent));
trace("mixed descent == 48px descent: " + (mixed.descent == ref.descent));
