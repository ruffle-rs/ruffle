package {
import flash.display.Sprite;
import flash.geom.Rectangle;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.Kerning;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var on:TextLine = lineWithKerning(Kerning.ON);
        var off:TextLine = lineWithKerning(Kerning.OFF);
        var shrink:Number = off.textWidth - on.textWidth;
        var onA:Rectangle = on.getAtomBounds(0);
        var offA:Rectangle = off.getAtomBounds(0);
        var onV:Rectangle = on.getAtomBounds(1);
        var offV:Rectangle = off.getAtomBounds(1);

        trace("kerning shrink positive: " + (shrink > 0));
        trace("left atom loses half: " + near(offA.width - onA.width, shrink / 2));
        trace("right atom starts halfway: " + near(onV.x - offV.x, -shrink / 2));
        trace("right atom keeps right edge: " + near(onV.x + onV.width, on.textWidth));
    }

    private function lineWithKerning(kerning:String):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "FTE GPOS Over Kern";
        var ef:ElementFormat = new ElementFormat(fd, 40);
        ef.kerning = kerning;
        return new TextBlock(new TextElement("AV", ef)).createTextLine(null, 10000);
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.08;
    }
}
}
