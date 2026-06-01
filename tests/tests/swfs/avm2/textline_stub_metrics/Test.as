package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="100", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var fmt:ElementFormat = new ElementFormat(fd, 14);
        var narrow:TextLine = new TextBlock(new TextElement("i", fmt))
            .createTextLine(null, 500);
        var text:TextLine = new TextBlock(new TextElement("Hello", fmt))
            .createTextLine(null, 500);

        trace("narrow.textWidth: " + rounded(narrow.textWidth));
        trace("text.textWidth: " + rounded(text.textWidth));
        trace("narrow.textHeight: " + rounded(narrow.textHeight));
        trace("text.textHeight: " + rounded(text.textHeight));
    }

    private function rounded(value:Number):Number {
        return Math.round(value * 100) / 100;
    }
}
}
