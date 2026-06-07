package {
import flash.display.Sprite;
import flash.geom.Rectangle;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// Inter-glyph spacing: ElementFormat.kerning and trackingLeft/trackingRight.
// Tracking is extra advance (in pixels) added left/right of every glyph;
// kerning pulls specific pairs together. Both change TextLine.textWidth and
// the atom positions. Each variant is rendered so the width change is visible.
public class Test extends Sprite {
    private var penY:Number = 6;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        sp("plain", "off", 0, 0);
        sp("kerning on", "on", 0, 0);
        sp("trackingRight +2", "off", 0, 2);
        sp("trackingRight -1", "off", 0, -1);
        sp("trackingLeft +3", "off", 3, 0);
        sp("tracking both 1+1", "off", 1, 1);
    }

    private function sp(label:String, kerning:String, trackLeft:Number, trackRight:Number):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var ef:ElementFormat = new ElementFormat(fd, 20);
        ef.kerning = kerning;
        ef.trackingLeft = trackLeft;
        ef.trackingRight = trackRight;
        var tb:TextBlock = new TextBlock(new TextElement("AVATAR Wave", ef));
        var tl:TextLine = tb.createTextLine(null, 10000);
        var lastRight:Number = 0;
        if (tl.atomCount > 0) {
            var b:Rectangle = tl.getAtomBounds(tl.atomCount - 1);
            lastRight = b.x + b.width;
        }
        trace(label
            + ": textWidth=" + tl.textWidth.toFixed(2)
            + " atomCount=" + tl.atomCount
            + " atom1.x=" + tl.getAtomBounds(1).x.toFixed(2)
            + " lastAtom.right=" + lastRight.toFixed(2));
        tl.x = 8;
        tl.y = penY + tl.ascent;
        addChild(tl);
        penY += tl.ascent + tl.descent + 12;
    }
}
}
