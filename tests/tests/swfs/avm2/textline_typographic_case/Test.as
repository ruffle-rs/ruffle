package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.LigatureLevel;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;
import flash.text.engine.TypographicCase;

// ElementFormat.typographicCase (caps / small-caps / upper / lower) and
// ligatureLevel are OpenType feature selections that change glyph selection
// and so TextLine.textWidth / atomCount. Each variant is rendered.
public class Test extends Sprite {
    private var penY:Number = 6;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        tcase("default", TypographicCase.DEFAULT);
        tcase("caps", TypographicCase.CAPS);
        tcase("smallCaps", TypographicCase.SMALL_CAPS);
        tcase("uppercase", TypographicCase.UPPERCASE);
        tcase("lowercase", TypographicCase.LOWERCASE);

        lig("none", LigatureLevel.NONE);
        lig("minimum", LigatureLevel.MINIMUM);
        lig("common", LigatureLevel.COMMON);
    }

    private function tcase(label:String, value:String):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var ef:ElementFormat = new ElementFormat(fd, 18);
        ef.typographicCase = value;
        var tb:TextBlock = new TextBlock(new TextElement("Multics 1965", ef));
        show("case " + label, tb.createTextLine(null, 10000));
    }

    private function lig(label:String, value:String):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_serif";
        var ef:ElementFormat = new ElementFormat(fd, 24);
        ef.ligatureLevel = value;
        var tb:TextBlock = new TextBlock(new TextElement("office waffle", ef));
        show("ligature " + label, tb.createTextLine(null, 10000));
    }

    private function show(label:String, tl:TextLine):void {
        trace(label
            + ": textWidth=" + tl.textWidth.toFixed(2)
            + " atomCount=" + tl.atomCount);
        tl.x = 8;
        tl.y = penY + tl.ascent;
        addChild(tl);
        penY += tl.ascent + tl.descent + 6;
    }
}
}
