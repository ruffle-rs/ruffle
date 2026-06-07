package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.FontPosture;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var regular:TextLine = lineWithPosture(FontPosture.NORMAL);
        var italic:TextLine = lineWithPosture(FontPosture.ITALIC);

        trace("italic changes width: " + (Math.abs(italic.textWidth - regular.textWidth) > 0.1));
        trace("italic keeps atoms: " + (italic.atomCount == regular.atomCount));
        trace("italic keeps text length: " + (italic.rawTextLength == regular.rawTextLength));
    }

    private function lineWithPosture(posture:String):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Serif";
        fd.fontPosture = posture;
        return new TextBlock(
            new TextElement("affine minimum", new ElementFormat(fd, 28))
        ).createTextLine(null, 10000);
    }
}
}
