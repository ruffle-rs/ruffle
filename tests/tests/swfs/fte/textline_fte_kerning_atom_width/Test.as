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
        var onLast:Rectangle = on.getAtomBounds(on.atomCount - 1);
        var offLast:Rectangle = off.getAtomBounds(off.atomCount - 1);

        trace("line advance changes: " + (Math.abs((onLast.x + onLast.width) - (offLast.x + offLast.width)) > 0.1));
        trace("textWidth matches atom advance: " + near(on.textWidth, onLast.x + onLast.width));
    }

    private function lineWithKerning(kerning:String):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var ef:ElementFormat = new ElementFormat(fd, 48);
        ef.kerning = kerning;
        return new TextBlock(new TextElement("AV", ef)).createTextLine(null, 10000);
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.01;
    }
}
}
