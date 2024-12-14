package {

import flash.display.InteractiveObject;
import flash.display.MovieClip;
import flash.display.Sprite;
import flash.events.Event;
import flash.events.KeyboardEvent;
import flash.events.MouseEvent;

[SWF(width="50", height="50", backgroundColor="#000000")]
public class Test extends MovieClip {
    private var clip:Sprite;

    public function Test() {
        super();

        this.clip = new Sprite();
        this.clip.x = 15;
        this.clip.y = 15;
        this.clip.graphics.beginFill(0xFF00FF);
        this.clip.graphics.drawRect(0, 0, 20, 20);
        this.clip.name = "clip";
        this.clip.tabEnabled = true;
        this.clip.tabIndex = 1;

        this.stage.addChild(this.clip);

        stage.addEventListener("keyDown", function(evt:KeyboardEvent):void {
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
                trace("  " + obj.name + ", " + evt.target.name + ": " + str);
            }
        }

        for each (var obj: InteractiveObject in [
            clip, stage
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
}
}
