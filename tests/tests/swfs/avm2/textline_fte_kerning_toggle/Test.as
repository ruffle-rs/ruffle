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

        trace("kerning changes width: " + (Math.abs(on.textWidth - off.textWidth) > 0.1));
        trace("kerning tightens AV: " + (on.textWidth < off.textWidth));
        trace("kerning keeps atoms: " + (on.atomCount == off.atomCount));
    }

    private function lineWithKerning(kerning:String):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var ef:ElementFormat = new ElementFormat(fd, 48);
        ef.kerning = kerning;
        return new TextBlock(new TextElement("AVAV", ef)).createTextLine(null, 10000);
    }
}
}
