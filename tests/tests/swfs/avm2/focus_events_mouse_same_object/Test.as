package {

import flash.display.InteractiveObject;
import flash.display.MovieClip;
import flash.display.Sprite;
import flash.events.Event;
import flash.events.KeyboardEvent;
import flash.events.MouseEvent;

[SWF(width="200", height="200")]
public class Test extends MovieClip {
    private var spriteA: Sprite;

    private var preventDefault: Boolean = false;

    public function Test() {
        super();

        spriteA = newSprite();
        spriteA.name = "spriteA";
        spriteA.x = 0;
        spriteA.y = 100;

        spriteA.tabEnabled = true;
        spriteA.tabIndex = 1;

        stage.addChild(spriteA);

        stage.addEventListener("keyDown", function(evt:KeyboardEvent):void {
            if (evt.keyCode == 27) {
                trace("Escape pressed");
                preventDefault = true;
                stage.focus = null;
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
                trace("  " + obj.name + ", " + evt.target.name + ": " + str);
                if (preventDefault && obj != stage) {
                    evt.preventDefault();
                }
            }
        }

        for each (var obj: InteractiveObject in [spriteA, stage]) {
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

    private function newSprite(): Sprite {
        var s:Sprite = new Sprite();
        s.graphics.beginFill(0x00FFFF);
        s.graphics.drawRect(0, 0, 100, 100);
        s.graphics.endFill();
        return s;
    }
}
}
