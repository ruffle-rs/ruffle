package {
import flash.display.Sprite;
import flash.ui.ContextMenu;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// TextLine rejects a handful of inherited InteractiveObject setters with Error
// #2181. Results traced for the Flash Player comparison (the real pass/fail);
// the stage shows a short description and a self-checked PASS/FAIL verdict.
public class Test extends Sprite {
    private var tl:TextLine;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        tl = new TextBlock(new TextElement("test", new ElementFormat(fd, 20)))
            .createTextLine(null, 200);

        var ok:Boolean = true;
        ok = probe("contextMenu", function():void { tl.contextMenu = new ContextMenu(); }) && ok;
        trace("contextMenu = " + tl.contextMenu);
        ok = probe("focusRect", function():void { tl.focusRect = true; }) && ok;
        trace("focusRect = " + tl.focusRect);
        ok = probe("tabChildren", function():void { tl.tabChildren = true; }) && ok;
        trace("tabChildren = " + tl.tabChildren);
        ok = probe("tabEnabled", function():void { tl.tabEnabled = true; }) && ok;
        trace("tabEnabled = " + tl.tabEnabled);
        ok = probe("tabIndex", function():void { tl.tabIndex = 3; }) && ok;
        trace("tabIndex = " + tl.tabIndex);

        verdict("TextLine rejects inherited InteractiveObject setters", ok);
    }

    // Returns true when the setter threw Error #2181, as expected.
    private function probe(name:String, setter:Function):Boolean {
        var threw:Boolean = false;
        try {
            setter();
            trace(name + " set: no error");
        } catch (error:*) {
            trace(name + " set: throws #" + error.errorID);
            threw = error.errorID == 2181;
        }
        return threw;
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
