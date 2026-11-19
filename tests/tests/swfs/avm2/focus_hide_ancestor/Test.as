package {
import flash.display.DisplayObject;
import flash.display.DisplayObjectContainer;
import flash.display.InteractiveObject;
import flash.display.SimpleButton;
import flash.display.Sprite;
import flash.display.Stage;
import flash.events.Event;
import flash.events.FocusEvent;
import flash.text.TextField;
import flash.text.TextFieldType;

// Hiding the focused object or one of its ancestors drops the focus.
[SWF(width="100", height="100", backgroundColor="#000000")]
public class Test extends Sprite {
    private var st:Stage;

    public function Test() {
        st = stage;
        var p:Sprite;
        var q:Sprite;
        var g:Sprite;
        var b:InteractiveObject;
        var o:DisplayObject;
        header("focused button hidden");
        p = holder("p", this);
        b = button("b", p);
        focus(b);
        b.visible = false;
        after();
        cleanup([p]);

        header("parent of the focused button hidden");
        p = holder("p", this);
        b = button("b", p);
        focus(b);
        p.visible = false;
        after();
        p.visible = true;
        trace("After showing the parent again: " + describe(st.focus));
        cleanup([p]);

        header("grandparent of the focused button hidden");
        g = holder("g", this);
        p = holder("p", g);
        b = button("b", p);
        focus(b);
        g.visible = false;
        after();
        cleanup([g]);

        header("parent of a focused input text field hidden");
        p = holder("p", this);
        b = text("t", p);
        focus(b);
        p.visible = false;
        after();
        cleanup([p]);

        header("parent of a focused off-stage button hidden");
        p = holder("p", null);
        b = button("b", p);
        focus(b);
        p.visible = false;
        after();
        cleanup([]);

        header("focus set while the parent is hidden");
        p = holder("p", this);
        b = button("b", p);
        p.visible = false;
        focus(b);
        cleanup([p]);

        header("already hidden parent hidden again");
        p = holder("p", this);
        b = button("b", p);
        p.visible = false;
        focus(b);
        p.visible = false;
        after();
        cleanup([p]);

        header("already hidden button hidden again");
        p = holder("p", this);
        b = button("b", p);
        b.visible = false;
        focus(b);
        b.visible = false;
        after();
        cleanup([p]);

        header("visible parent of the focused button made visible");
        p = holder("p", this);
        b = button("b", p);
        focus(b);
        p.visible = true;
        after();
        cleanup([p]);

        header("sibling of the focused button hidden");
        p = holder("p", this);
        o = button("other", p);
        b = button("b", p);
        focus(b);
        o.visible = false;
        after();
        cleanup([p]);
    }

    private function header(name:String):void {
        trace("===== " + name);
    }

    private function describe(obj:Object):String {
        if (obj == null) {
            return "null";
        }
        if (obj == st) {
            return "stage";
        }
        return obj.name;
    }

    private function holder(name:String, parent:DisplayObjectContainer):Sprite {
        var s:Sprite = new Sprite();
        s.name = name;
        if (parent != null) {
            parent.addChild(s);
        }
        return s;
    }

    private function watch(obj:InteractiveObject):InteractiveObject {
        for each (var type:String in [FocusEvent.FOCUS_IN, FocusEvent.FOCUS_OUT, Event.ADDED, Event.ADDED_TO_STAGE, Event.REMOVED, Event.REMOVED_FROM_STAGE]) {
            obj.addEventListener(type, onEvent);
        }
        return obj;
    }

    private function onEvent(e:Event):void {
        var related:String = "";
        if (e is FocusEvent) {
            related = ", relatedObject: " + describe(FocusEvent(e).relatedObject);
        }
        trace("  " + e.type + " on " + describe(e.target) + related + ", focus: " + describe(st.focus));
    }

    private function button(name:String, parent:DisplayObjectContainer):InteractiveObject {
        var b:SimpleButton = new SimpleButton();
        b.name = name;
        parent.addChild(b);
        return name == "other" ? b : watch(b);
    }

    private function text(name:String, parent:DisplayObjectContainer):InteractiveObject {
        var t:TextField = new TextField();
        t.name = name;
        t.type = TextFieldType.INPUT;
        parent.addChild(t);
        return watch(t);
    }

    private function sprite(name:String, parent:DisplayObjectContainer):InteractiveObject {
        var s:Sprite = new Sprite();
        s.name = name;
        parent.addChild(s);
        return watch(s);
    }

    private function focus(obj:InteractiveObject):void {
        st.focus = obj;
        trace("Focus set: " + describe(st.focus));
    }

    private function after():void {
        trace("After: " + describe(st.focus));
    }

    private function cleanup(objects:Array):void {
        trace("Cleanup");
        st.focus = null;
        for each (var obj:DisplayObject in objects) {
            if (obj.parent != null) {
                obj.parent.removeChild(obj);
            }
        }
    }
}
}
