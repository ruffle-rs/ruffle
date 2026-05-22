package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// CJK text laid out through flash.text.engine with Noto Sans CJK. Japanese,
// Chinese and Korean lines exercise glyph lookup, metrics and rendering for
// non-Latin scripts; each line's metrics are traced for the Flash Player
// comparison and the lines are added to the stage for the visual check.
public class Test extends Sprite {
    private var penY:Number = 12;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        show("日本語");
        show("中文字");
        show("한국어");
    }

    private function show(text:String):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Noto Sans CJK JP";
        var ef:ElementFormat = new ElementFormat(fd, 48);
        var tl:TextLine = new TextBlock(new TextElement(text, ef))
            .createTextLine(null, 480);
        trace(text + ": atomCount=" + tl.atomCount
            + " textWidth=" + tl.textWidth.toFixed(2)
            + " ascent=" + tl.ascent.toFixed(2));
        tl.x = 30;
        tl.y = penY + tl.ascent;
        addChild(tl);
        penY += 110;
    }
}
}
