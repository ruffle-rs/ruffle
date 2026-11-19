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

// Removing the focused object clears the focus without focus events, after the removal events.
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
        header("removeChild of the focused button");
        p = holder("p", this);
        b = button("b", p);
        focus(b);
        p.removeChild(b);
        after();
        cleanup([p]);

        header("removeChildAt of the focused button");
        p = holder("p", this);
        button("other", p);
        b = button("b", p);
        focus(b);
        p.removeChildAt(1);
        after();
        cleanup([p]);

        header("removeChildren on the focused button's parent");
        p = holder("p", this);
        b = button("b", p);
        focus(b);
        p.removeChildren();
        after();
        cleanup([p]);

        header("removeChild of a focused input text field");
        p = holder("p", this);
        b = text("t", p);
        focus(b);
        p.removeChild(b);
        after();
        cleanup([p]);

        header("removeChild of a focused sprite");
        p = holder("p", this);
        b = sprite("s", p);
        focus(b);
        p.removeChild(b);
        after();
        cleanup([p]);

        header("removeChild of a focused button from an off-stage parent");
        p = holder("p", null);
        b = button("b", p);
        focus(b);
        p.removeChild(b);
        after();
        cleanup([]);
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
