package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var block:TextBlock = new TextBlock(
            new TextElement("target", new ElementFormat(fd, 20))
        );

        try {
            var line:TextLine = block.recreateTextLine(null, null, 1000);
            trace("null target returned: " + line);
        } catch (e:Error) {
            trace("null target error: " + e.errorID);
        }
    }
}
}
