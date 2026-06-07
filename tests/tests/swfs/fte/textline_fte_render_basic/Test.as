package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 200, 100);
        graphics.endFill();

        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var ef:ElementFormat = new ElementFormat(fd, 36, 0x000000);
        var line:TextLine = new TextBlock(new TextElement("FTE", ef)).createTextLine(null, 180);
        line.x = 20;
        line.y = 55;
        addChild(line);

        trace("render line added: " + (line.parent == this));
    }
}
}
