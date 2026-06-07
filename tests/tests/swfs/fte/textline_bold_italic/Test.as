package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.FontPosture;
import flash.text.engine.FontWeight;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// FontDescription.fontWeight and fontPosture select the bold / italic device
// font variants. Each of the four combinations is laid out and rendered so
// the weight and slant are visible.
public class Test extends Sprite {
    private var penY:Number = 6;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();
        show("regular", FontWeight.NORMAL, FontPosture.NORMAL);
        show("bold", FontWeight.BOLD, FontPosture.NORMAL);
        show("italic", FontWeight.NORMAL, FontPosture.ITALIC);
        show("bold italic", FontWeight.BOLD, FontPosture.ITALIC);
    }

    private function show(label:String, weight:String, posture:String):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        fd.fontWeight = weight;
        fd.fontPosture = posture;
        var ef:ElementFormat = new ElementFormat(fd, 28);
        var tb:TextBlock = new TextBlock(new TextElement("Multics " + label, ef));
        var tl:TextLine = tb.createTextLine(null, 10000);
        trace(label + ": textWidth=" + tl.textWidth.toFixed(2)
            + " ascent=" + tl.ascent.toFixed(2)
            + " descent=" + tl.descent.toFixed(2)
            + " atomCount=" + tl.atomCount);
        tl.x = 12;
        tl.y = penY + tl.ascent;
        addChild(tl);
        penY += tl.ascent + tl.descent + 6;
    }
}
}
