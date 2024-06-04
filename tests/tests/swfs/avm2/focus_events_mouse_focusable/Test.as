package {

import flash.display.DisplayObject;
import flash.display.InteractiveObject;
import flash.display.MovieClip;
import flash.display.SimpleButton;
import flash.display.Sprite;
import flash.events.Event;
import flash.events.FocusEvent;
import flash.events.MouseEvent;
import flash.text.TextField;

[SWF(width="1000", height="500")]
public class Test extends MovieClip {
    private var currentColor:int = 0;
    private var colors:Array = [
        0xFF0000,
        0x00FF00,
        0x0000FF,
        0x00FFFF,
        0xFF00FF,
        0xFFFF00
    ];

    private var objectId:int = 0;
    private var objects:Array = [];

    public function Test() {
        super();

        objects.push(newSprite(false, false, false)); // 0
        objects.push(newSprite(false, true, false));
        objects.push(newSprite(false, false, true));
        objects.push(newSprite(false, true, true));
        objects.push(newSprite(true, false, false));
        objects.push(newSprite(true, true, false));
        objects.push(newSprite(true, false, true));
        objects.push(newSprite(true, true, true));
        objects.push(newTextField(false, false, false));
        objects.push(newTextField(false, false, true));
        objects.push(newTextField(false, true, false)); // 10
        objects.push(newTextField(false, true, true));
        objects.push(newTextField(true, false, false));
        objects.push(newTextField(true, false, true));
        objects.push(newTextField(true, true, false));
        objects.push(newTextField(true, true, true));
        objects.push(newButton(false, false, false));
        objects.push(newButton(false, true, false));
        objects.push(newButton(false, false, true));
        objects.push(newButton(false, true, true));
        objects.push(newButton(true, false, false)); // 20
        objects.push(newButton(true, true, false));
        objects.push(newButton(true, false, true));
        objects.push(newButton(true, true, true));
        objects.push(newMovieClip(false, false, false, false));
        objects.push(newMovieClip(false, false, false, true));
        objects.push(newMovieClip(false, false, true, false));
        objects.push(newMovieClip(false, false, true, true));
        objects.push(newMovieClip(false, true, false, false));
        objects.push(newMovieClip(false, true, false, true));
        objects.push(newMovieClip(false, true, true, false)); // 30
        objects.push(newMovieClip(false, true, true, true));
        objects.push(newMovieClip(true, false, false, false));
        objects.push(newMovieClip(true, false, false, true));
        objects.push(newMovieClip(true, false, true, false));
        objects.push(newMovieClip(true, false, true, true));
        objects.push(newMovieClip(true, true, false, false));
        objects.push(newMovieClip(true, true, false, true));
        objects.push(newMovieClip(true, true, true, false));
        objects.push(newMovieClip(true, true, true, true));

        var x = 0;
        var y = 100;
        for each (var object:InteractiveObject in objects) {
            object.x = x;
            object.y = y;

            x += 100;
            if (x >= 1000) {
                x = 0;
                y += 100;
            }

            addChild(object);
        }

        function eventListener(obj:InteractiveObject):Function {
            return function (evt:Event):void {
                var str;
                if (evt is MouseEvent) {
                    str = evt.formatToString(
                            "MouseEvent", "type", "cancelable", "eventPhase",
                            "relatedObject", "ctrlKey", "altKey", "shiftKey");
                } else {
                    str = evt.toString();
                }
                if (evt is FocusEvent && FocusEvent(evt).relatedObject) {
                    str += ", relatedObjectName=" + FocusEvent(evt).relatedObject.name;
                }
                trace(obj.name + ", " + evt.target.name + ": " + str + ", focus: " + objectToString(stage.focus));
            }
        }

        for each (var obj:InteractiveObject in objects) {
            obj.addEventListener("focusIn", eventListener(obj));
            obj.addEventListener("focusOut", eventListener(obj));
            obj.addEventListener("mouseFocusChange", eventListener(obj));
            obj.addEventListener("keyFocusChange", eventListener(obj));
            obj.addEventListener("click", eventListener(obj));
        }
    }

    private function objectToString(obj: DisplayObject): String {
        return "" + obj + ((obj != null) ? " (" + obj.name + ")" : "");
    }

    private function newSprite(tabEnabled:Boolean, buttonMode:Boolean, handCursor:Boolean):Sprite {
        var s:Sprite = new Sprite();
        var color:int = nextColor();
        s.graphics.beginFill(color);
        s.graphics.drawRect(0, 0, 100, 100);
        s.graphics.endFill();
        s.buttonMode = buttonMode;
        s.useHandCursor = handCursor;
        s.tabEnabled = tabEnabled;
        s.name = "sprite" + (objectId++);
        return s;
    }

    private function newMovieClip(tabEnabled:Boolean, enabled:Boolean, buttonMode:Boolean, handCursor:Boolean):MovieClip {
        var mc:MovieClip = new MovieClip();
        var color:int = nextColor();
        mc.enabled = enabled;
        mc.buttonMode = buttonMode;
        mc.useHandCursor = handCursor;
        mc.graphics.beginFill(color);
        mc.graphics.drawRect(0, 0, 100, 100);
        mc.graphics.endFill();
        mc.tabEnabled = tabEnabled;
        mc.name = "clip" + (objectId++);
        return mc;
    }

    private function newTextField(tabEnabled:Boolean, input:Boolean, selectable:Boolean):TextField {
        var tf:TextField = new TextField();
        tf.type = input ? "input" : "dynamic";
        tf.border = true;
        tf.width = 100;
        tf.height = 100;
        tf.selectable = selectable;
        var color:int = nextColor();
        tf.borderColor = color;
        tf.tabEnabled = tabEnabled;
        tf.name = "text" + (objectId++);
        return tf;
    }

    private function newButton(tabEnabled:Boolean, enabled:Boolean, handCursor:Boolean):SimpleButton {
        var b:SimpleButton = new SimpleButton();
        var color:int = nextColor();
        var state:ButtonDisplayState = new ButtonDisplayState(color, 100);
        b.downState = state;
        b.overState = state;
        b.upState = state;
        b.hitTestState = state;
        b.enabled = enabled;
        b.useHandCursor = handCursor;
        b.tabEnabled = tabEnabled;
        b.name = "button" + (objectId++);
        return b;
    }

    private function nextColor():int {
        return colors[currentColor++ % 6];
    }
}
}
