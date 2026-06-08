package {
import flash.display.Sprite;
import flash.geom.Rectangle;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var line:TextLine = new TextBlock(
            new TextElement("HIT", new ElementFormat(fd, 24))
        ).createTextLine(null, 400);
        line.x = 30;
        line.y = 80;
        addChild(line);

        for (var i:int = 0; i < line.atomCount; i++) {
            var bounds:Rectangle = line.getAtomBounds(i);
            var stageX:Number = line.x + bounds.x + bounds.width / 2;
            var stageY:Number = line.y + bounds.y + bounds.height / 2;
            trace("atom " + i + " hit: " + line.getAtomIndexAtPoint(stageX, stageY));
        }
    }
}
}
