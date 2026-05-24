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
            new TextElement("release data", new ElementFormat(fd, 20))
        );
        var line:TextLine = block.createTextLine(null, 1000);

        trace("line before release: " + (line != null));
        try {
            block.releaseLineCreationData();
            trace("release callable: true");
        } catch (e:Error) {
            trace("release error: " + e.errorID);
        }

        try {
            trace("dump value: " + block.dump());
        } catch (e:Error) {
            trace("dump error: " + e.errorID);
        }

        trace("line after release: " + (block.firstLine == line));
    }
}
}
