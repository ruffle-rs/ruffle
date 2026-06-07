package {
import flash.display.Sprite;
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

        trace("kerning enabled: " + (shrink > 0));
        trace("uses gpos not kern: " + (shrink > 0.5 && shrink < 2.0));
        trace("legacy kern would be huge: " + (shrink < 5.0));
    }

    private function lineWithKerning(kerning:String):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "FTE GPOS Over Kern";
        var ef:ElementFormat = new ElementFormat(fd, 40);
        ef.kerning = kerning;
        return new TextBlock(new TextElement("AV", ef)).createTextLine(null, 10000);
    }
}
}
