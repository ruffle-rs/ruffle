package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// The _sans, _serif and _typewriter device-font aliases resolve to three
// distinct fonts, each with its own metrics. The same string is laid out in
// each and rendered, so the differing widths and shapes are visible.
public class Test extends Sprite {
    private var penY:Number = 6;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();
        show("_sans");
        show("_serif");
        show("_typewriter");
    }

    private function show(fontName:String):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = fontName;
        var ef:ElementFormat = new ElementFormat(fd, 26);
        var tb:TextBlock = new TextBlock(new TextElement("Multics " + fontName, ef));
        var tl:TextLine = tb.createTextLine(null, 10000);
        trace(fontName + ": textWidth=" + tl.textWidth.toFixed(2)
            + " ascent=" + tl.ascent.toFixed(2)
            + " descent=" + tl.descent.toFixed(2)
            + " atomCount=" + tl.atomCount);
        tl.x = 12;
        tl.y = penY + tl.ascent;
        addChild(tl);
        penY += tl.ascent + tl.descent + 8;
    }
}
}
