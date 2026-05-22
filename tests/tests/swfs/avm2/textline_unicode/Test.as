package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// Non-ASCII content: accented Latin letters and typographic punctuation are
// shaped, measured and rendered like any other run. Each line is rendered so
// the diacritics are visible.
public class Test extends Sprite {
    private var penY:Number = 6;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();
        show(0, "café éèêë");
        show(1, "niño jalapeño");
        show(2, "Ångström über");
        show(3, "quotes ‘x’ “y” — dash");
    }

    private function show(n:int, txt:String):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var ef:ElementFormat = new ElementFormat(fd, 24);
        var tb:TextBlock = new TextBlock(new TextElement(txt, ef));
        var tl:TextLine = tb.createTextLine(null, 10000);
        trace("line " + n + ": rawTextLength=" + tl.rawTextLength
            + " atomCount=" + tl.atomCount
            + " textWidth=" + tl.textWidth.toFixed(2));
        tl.x = 12;
        tl.y = penY + tl.ascent;
        addChild(tl);
        penY += tl.ascent + tl.descent + 8;
    }
}
}
