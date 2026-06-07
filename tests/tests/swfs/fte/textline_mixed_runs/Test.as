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

// A GroupElement composes several TextElements with different ElementFormats
// into one TextBlock. The resulting TextLine's ascent/descent take the max
// over all runs, atoms tile the runs in order, each run keeps its own font,
// size and color. Rendered so the mixed run is visible.
public class Test extends Sprite {
    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var elements:Vector.<ContentElement> = new Vector.<ContentElement>();
        elements.push(new TextElement("Big ", fmt("_sans", 30, 0x000000)));
        elements.push(new TextElement("tiny ", fmt("_sans", 10, 0xcc0000)));
        elements.push(new TextElement("serif", fmt("_serif", 20, 0x0000cc)));

        var tb:TextBlock = new TextBlock(new GroupElement(elements));
        var tl:TextLine = tb.createTextLine(null, 10000);

        trace("mixed run line:"
            + " ascent=" + tl.ascent.toFixed(2)
            + " descent=" + tl.descent.toFixed(2)
            + " textHeight=" + tl.textHeight.toFixed(2)
            + " textWidth=" + tl.textWidth.toFixed(2)
            + " atomCount=" + tl.atomCount);
        for (var i:int = 0; i < tl.atomCount; i++) {
            var b:Rectangle = tl.getAtomBounds(i);
            trace("  atom " + i
                + " x=" + b.x.toFixed(2)
                + " width=" + b.width.toFixed(2)
                + " top=" + b.y.toFixed(2)
                + " height=" + b.height.toFixed(2));
        }

        tl.x = 20;
        tl.y = 60 + tl.ascent;
        addChild(tl);
    }

    private function fmt(fontName:String, size:Number, color:uint):ElementFormat {
        var fd:FontDescription = new FontDescription();
        fd.fontName = fontName;
        var ef:ElementFormat = new ElementFormat(fd, size);
        ef.color = color;
        return ef;
    }
}
}
