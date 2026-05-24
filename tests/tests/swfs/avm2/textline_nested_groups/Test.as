package {
import flash.display.Sprite;
import flash.geom.Rectangle;
import flash.text.engine.ContentElement;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.GroupElement;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// GroupElement nesting: a GroupElement may contain other GroupElements. The
// content tree is flattened to per-run formats in document order, so a run
// inside a nested group keeps its own size and color. Rendered in colour.
public class Test extends Sprite {
    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var inner:Vector.<ContentElement> = new Vector.<ContentElement>();
        inner.push(new TextElement("red", fmt(0xcc0000, 22)));
        inner.push(new TextElement("blue", fmt(0x0000cc, 22)));
        var innerGroup:GroupElement = new GroupElement(inner);

        var outer:Vector.<ContentElement> = new Vector.<ContentElement>();
        outer.push(new TextElement("A ", fmt(0x000000, 30)));
        outer.push(innerGroup);
        outer.push(new TextElement(" Z", fmt(0x000000, 30)));

        var tb:TextBlock = new TextBlock(new GroupElement(outer));
        var tl:TextLine = tb.createTextLine(null, 10000);

        trace("nested group: textWidth=" + tl.textWidth.toFixed(2)
            + " atomCount=" + tl.atomCount
            + " ascent=" + tl.ascent.toFixed(2));
        var i:int = 0;
        while (i < tl.atomCount) {
            var b:Rectangle = tl.getAtomBounds(i);
            trace("  atom " + i + " x=" + b.x.toFixed(2) + " width=" + b.width.toFixed(2));
            i++;
        }

        tl.x = 12;
        tl.y = 40 + tl.ascent;
        addChild(tl);
    }

    private function fmt(color:uint, size:Number):ElementFormat {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var ef:ElementFormat = new ElementFormat(fd, size);
        ef.color = color;
        return ef;
    }
}
}
