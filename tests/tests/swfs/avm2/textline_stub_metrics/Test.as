package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="100", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        fd.fontLookup = FontLookup.DEVICE;
        var fmt:ElementFormat = new ElementFormat(fd, 14);
        var narrow:TextLine = new TextBlock(new TextElement("i", fmt))
            .createTextLine(null, 500);
        var text:TextLine = new TextBlock(new TextElement("Hello", fmt))
            .createTextLine(null, 500);

        trace("textWidth follows glyph advance (these are still stubs): " + (text.textWidth > narrow.textWidth));
        trace("textHeight is stable for same format (these are still stubs): " + near(text.textHeight, narrow.textHeight));
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.01;
    }
}
}
