package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// Exercises TextBlock.createTextLine / recreateTextLine state. Every result is
// traced for the Flash Player trace comparison (the real pass/fail); the stage
// shows a short description and a self-checked PASS/FAIL verdict.
public class Test extends Sprite {
    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var fmt:ElementFormat = new ElementFormat(fd, 14);
        var ok:Boolean = true;

        var tb1:TextBlock = new TextBlock(new TextElement("Hello", fmt));
        var line1:TextLine = tb1.createTextLine(null, 500);
        trace("line1 != null: " + (line1 != null));
        trace("rawTextLength: " + line1.rawTextLength);
        trace("textBlockBeginIndex: " + line1.textBlockBeginIndex);
        trace("result: " + tb1.textLineCreationResult);
        ok = ok && line1 != null && line1.rawTextLength == 5
            && line1.textBlockBeginIndex == 0
            && tb1.textLineCreationResult == "success";

        var line1b:TextLine = tb1.createTextLine(line1, 500);
        trace("line1b == null: " + (line1b == null));
        trace("result after complete: " + tb1.textLineCreationResult);
        ok = ok && line1b == null && tb1.textLineCreationResult == "complete";

        var tb2:TextBlock = new TextBlock();
        var nullLine:TextLine = tb2.createTextLine(null, 500);
        trace("null content line: " + nullLine);
        ok = ok && nullLine == null;

        var tb3:TextBlock = new TextBlock(new TextElement("ABCDEF", fmt));
        var rl:TextLine = tb3.createTextLine(null, 500);
        trace("before recreate rawTextLength: " + rl.rawTextLength);
        var rl2:TextLine = tb3.recreateTextLine(rl, null, 500);
        trace("same object: " + (rl === rl2));
        trace("after recreate rawTextLength: " + rl2.rawTextLength);
        trace("after recreate beginIndex: " + rl2.textBlockBeginIndex);
        trace("result: " + tb3.textLineCreationResult);
        ok = ok && rl.rawTextLength == 6 && rl === rl2 && rl2.rawTextLength == 6
            && rl2.textBlockBeginIndex == 0 && tb3.textLineCreationResult == "success";

        var tb4:TextBlock = new TextBlock(new TextElement("Test", fmt));
        var firstBefore:Boolean = tb4.firstLine == null;
        trace("firstLine before: " + tb4.firstLine);
        var fl:TextLine = tb4.createTextLine(null, 500);
        trace("firstLine after: " + (tb4.firstLine === fl));
        ok = ok && firstBefore && tb4.firstLine === fl;

        verdict("TextBlock.createTextLine / recreateTextLine / firstLine", ok);
    }

    private function verdict(desc:String, ok:Boolean):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var df:ElementFormat = new ElementFormat(fd, 17);
        var d:TextLine = new TextBlock(new TextElement(desc, df))
            .createTextLine(null, 470);
        d.x = 20;
        d.y = 60 + d.ascent;
        addChild(d);
        var vf:ElementFormat = new ElementFormat(fd, 52);
        vf.color = ok ? 0x118811 : 0xCC1111;
        var v:TextLine = new TextBlock(new TextElement(ok ? "PASS" : "FAIL", vf))
            .createTextLine(null, 470);
        v.x = 20;
        v.y = 130 + v.ascent;
        addChild(v);
    }
}
}
