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
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var normal:TextLine = makeLine(0, 0x000000);
        normal.x = 30;
        normal.y = 90;
        addChild(normal);

        var shifted:TextLine = makeLine(-24, 0xCC0000);
        shifted.x = 30;
        shifted.y = 160;
        addChild(shifted);

        trace("metrics unchanged: " + near(normal.textHeight, shifted.textHeight));
        trace("shifted line added: " + (shifted.parent == this));
    }

    private function makeLine(shift:Number, color:uint):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var ef:ElementFormat = new ElementFormat(fd, 38, color);
        ef.baselineShift = shift;
        return new TextBlock(new TextElement("Shift", ef)).createTextLine(null, 300);
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.01;
    }
}
}
