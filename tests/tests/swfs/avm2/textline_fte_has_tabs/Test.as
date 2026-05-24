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
        fd.fontName = "_sans";
        var ef:ElementFormat = new ElementFormat(fd, 18);
        var withTab:TextLine = new TextBlock(new TextElement("a\tb", ef)).createTextLine(null, 400);
        var withoutTab:TextLine = new TextBlock(new TextElement("abc", ef)).createTextLine(null, 400);

        trace("with tab: " + withTab.hasTabs);
        trace("without tab: " + withoutTab.hasTabs);
    }
}
}
