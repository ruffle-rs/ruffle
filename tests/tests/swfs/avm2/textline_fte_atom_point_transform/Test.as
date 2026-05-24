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
        var holder:Sprite = new Sprite();
        holder.x = 40;
        holder.y = 20;
        holder.scaleX = 2;
        holder.scaleY = 1.5;
        addChild(holder);

        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var line:TextLine = new TextBlock(
            new TextElement("AB", new ElementFormat(fd, 24))
        ).createTextLine(null, 400);
        line.x = 7;
        line.y = 50;
        holder.addChild(line);

        var first:Rectangle = line.getAtomBounds(0);
        var second:Rectangle = line.getAtomBounds(1);
        trace("inside transformed first: " + line.getAtomIndexAtPoint(toStageX(line, first), toStageY(line, first)));
        trace("inside transformed second: " + line.getAtomIndexAtPoint(toStageX(line, second), toStageY(line, second)));
        trace("left outside: " + line.getAtomIndexAtPoint(holder.x, holder.y + line.y));
        trace("right outside: " + line.getAtomIndexAtPoint(holder.x + 400, holder.y + line.y));
    }

    private function toStageX(line:TextLine, r:Rectangle):Number {
        return 40 + (line.x + r.x + r.width / 2) * 2;
    }

    private function toStageY(line:TextLine, r:Rectangle):Number {
        return 20 + (line.y + r.y + r.height / 2) * 1.5;
    }
}
}
