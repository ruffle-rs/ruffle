package {

import flash.display.DisplayObject;
import flash.display.InteractiveObject;
import flash.display.MovieClip;
import flash.display.Sprite;
import flash.events.Event;
import flash.events.KeyboardEvent;
import flash.events.MouseEvent;

[SWF(width="300", height="200")]
public class Test extends MovieClip {
    private var sprite1: Sprite;
    private var sprite2: Sprite;
    private var sprite3: Sprite;

    private var logDirty: Boolean = true;

    public function Test() {
        super();

        sprite1 = newSprite(0xFF0000);
        sprite1.name = "sprite1";
        sprite1.x = 0;
        sprite1.y = 100;
        sprite1.tabEnabled = true;
        sprite1.tabIndex = 1;
        sprite2 = newSprite(0x00FF00);
        sprite2.name = "sprite2";
        sprite2.x = 100;
        sprite2.y = 100;
        sprite2.tabEnabled = true;
        sprite2.tabIndex = 2;
        sprite3 = newSprite(0x0000FF);
        sprite3.name = "sprite3";
        sprite3.x = 200;
        sprite3.y = 100;
        sprite3.tabEnabled = true;

        stage.addChild(sprite1);
        stage.addChild(sprite2);
        stage.addChild(sprite3);

        stage.addEventListener("keyDown", function(evt:KeyboardEvent):void {
            if (evt.keyCode == 27 && logDirty) {
                logDirty = false;
                trace("Escape pressed");
            }
            if (evt.keyCode == 9) {
                trace("Tab pressed");
            }
        });

        function eventListener(obj: InteractiveObject): Function {
            return function(evt: Event): void {
                var str;
                if (evt is MouseEvent) {
                    str = evt.formatToString(
                            "MouseEvent", "type", "cancelable", "eventPhase",
                            "relatedObject", "ctrlKey", "altKey", "shiftKey");
                } else {
                    str = evt.toString();
                }
                trace("  " + obj.name + ", " + evt.target.name + ": " + str + ", focus: " + objectToString(stage.focus));
                logDirty = true;
            }
        }

        for each (var obj: InteractiveObject in [
            sprite1, sprite2, sprite3,
            stage
        ]) {
            obj.addEventListener("focusIn", eventListener(obj));
            obj.addEventListener("focusOut", eventListener(obj));
            obj.addEventListener("mouseDown", eventListener(obj));
            obj.addEventListener("mouseUp", eventListener(obj));
            obj.addEventListener("click", eventListener(obj));
            obj.addEventListener("mouseFocusChange", eventListener(obj));
            obj.addEventListener("keyFocusChange", eventListener(obj));
            obj.addEventListener("rollOut", eventListener(obj));
            obj.addEventListener("rollOver", eventListener(obj));
            obj.addEventListener("mouseOver", eventListener(obj));
        }
    }

    private function objectToString(obj: DisplayObject): String {
        return "" + obj + ((obj != null) ? " (" + obj.name + ")" : "");
    }

    private function newSprite(color: int): Sprite {
        var s:Sprite = new Sprite();
        s.graphics.beginFill(color);
        s.graphics.drawRect(0, 0, 100, 100);
        s.graphics.endFill();
        return s;
    }
}
}
