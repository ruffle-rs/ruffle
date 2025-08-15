package {

import flash.display.DisplayObject;
import flash.display.InteractiveObject;
import flash.display.MovieClip;
import flash.display.Sprite;
import flash.events.Event;
import flash.events.FocusEvent;
import flash.events.KeyboardEvent;
import flash.ui.Keyboard;

[SWF(width="200", height="200")]
public class Test extends MovieClip {
    private var sprite1: Sprite;
    private var sprite2: Sprite;
    private var sprite3: Sprite;
    private var sprite4: Sprite;

    public function Test() {
        super();

        sprite1 = newSprite();
        sprite1.name = "sprite1";
        sprite1.x = 0;
        sprite1.y = 0;
        sprite1.tabEnabled = true;
        sprite2 = newSprite();
        sprite2.name = "sprite2";
        sprite2.x = 100;
        sprite2.y = 0;
        sprite2.tabEnabled = true;
        sprite3 = newSprite();
        sprite3.name = "sprite3";
        sprite3.x = 0;
        sprite3.y = 100;
        sprite3.tabEnabled = true;
        sprite4 = newSprite();
        sprite4.name = "sprite4";
        sprite4.x = 100;
        sprite4.y = 100;
        sprite4.tabEnabled = true;

        stage.addChild(sprite1);
        stage.addChild(sprite2);
        stage.addChild(sprite3);
        stage.addChild(sprite4);

        stage.addEventListener("keyDown", function(evt:KeyboardEvent):void {
            if (evt.keyCode == 27) {
                trace("Escape pressed");
                stage.focus = null;
            }
            if (evt.keyCode == Keyboard.LEFT) {
                trace("Pressed left");
            }
            if (evt.keyCode == Keyboard.RIGHT) {
                trace("Pressed right");
            }
            if (evt.keyCode == Keyboard.UP) {
                trace("Pressed up");
            }
            if (evt.keyCode == Keyboard.DOWN) {
                trace("Pressed down");
            }
        });

        function eventListener(obj: InteractiveObject): Function {
            return function(evt: Event): void {
                trace("  " + obj.name + ", " + evt.target.name + ": " + evt.toString() + ", focus: " + objectToString(stage.focus));
                if (obj == sprite4 && evt is FocusEvent && FocusEvent(evt).keyCode == Keyboard.LEFT) {
                    evt.preventDefault();
                }
                if (obj == sprite3 && evt is FocusEvent && FocusEvent(evt).keyCode == Keyboard.RIGHT) {
                    evt.preventDefault();
                }
            }
        }

        for each (var obj: InteractiveObject in [
            sprite1, sprite2, sprite3, sprite4,
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

    private function newSprite(): Sprite {
        var s:Sprite = new Sprite();
        s.graphics.beginFill(0x00FFFF);
        s.graphics.drawRect(10, 10, 80, 80);
        s.graphics.endFill();
        return s;
    }
}
}
