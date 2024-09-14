package {

import flash.display.DisplayObject;
import flash.display.InteractiveObject;
import flash.display.MovieClip;
import flash.display.SimpleButton;
import flash.display.Sprite;
import flash.events.Event;
import flash.events.KeyboardEvent;
import flash.events.MouseEvent;
import flash.text.TextField;
import flash.ui.Keyboard;

[SWF(width="800", height="200")]
public class Test extends MovieClip {
    private var sprite1: Sprite;
    private var sprite2: Sprite;
    private var mc1: MovieClip;
    private var mc2: MovieClip;
    private var text: TextField;
    private var button1: SimpleButton;
    private var button2: SimpleButton;
    private var guard: SimpleButton;

    public function Test() {
        super();

        sprite1 = newSprite();
        sprite1.name = "sprite1";
        sprite1.x = 0;
        sprite1.y = 100;
        sprite1.tabEnabled = true;
        sprite2 = newSprite();
        sprite2.name = "sprite2";
        sprite2.x = 100;
        sprite2.y = 100;
        mc1 = newMovieClip(false, true);
        mc1.name = "mc1";
        mc1.x = 200;
        mc1.y = 100;
        mc2 = newMovieClip(true, true);
        mc2.name = "mc2";
        mc2.x = 300;
        mc2.y = 100;
        mc2.tabEnabled = true;
        text = newTextField();
        text.name = "textA";
        text.x = 400;
        text.y = 100;
        button1 = newButton();
        button1.name = "button1";
        button1.x = 500;
        button1.y = 100;
        button1.tabEnabled = true;
        button2 = newButton();
        button2.name = "button2";
        button2.x = 600;
        button2.y = 100;
        guard = newButton();
        guard.name = "guard";
        guard.x = 700;
        guard.y = 100;

        stage.addChild(sprite1);
        stage.addChild(sprite2);
        stage.addChild(mc1);
        stage.addChild(mc2);
        stage.addChild(text);
        stage.addChild(button1);
        stage.addChild(button2);

        stage.addEventListener("keyDown", function(evt:KeyboardEvent):void {
            if (evt.keyCode == 9) {
                trace("Tab pressed");
            }
            if (evt.keyCode == 27) {
                trace("Escape pressed");
                stage.focus = null;
            }
            if (evt.keyCode == Keyboard.NUMBER_1) {
                trace("Adding guard");
                stage.addChild(guard);
            }
        });

        function eventListener(obj: InteractiveObject, preventDefault:Boolean): Function {
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
                if (preventDefault) {
                    evt.preventDefault();
                }
            }
        }

        for each (var obj: InteractiveObject in [
            sprite1, sprite2, mc1, mc2, text, button1, button2,
            stage
        ]) {
            obj.addEventListener("focusIn", eventListener(obj, false));
            obj.addEventListener("focusOut", eventListener(obj, false));
            obj.addEventListener("mouseDown", eventListener(obj, false));
            obj.addEventListener("mouseUp", eventListener(obj, false));
            obj.addEventListener("click", eventListener(obj, false));
            obj.addEventListener("mouseFocusChange", eventListener(obj, false));
            obj.addEventListener("keyFocusChange", eventListener(obj, false));
            obj.addEventListener("rollOut", eventListener(obj, false));
            obj.addEventListener("rollOver", eventListener(obj, false));
            obj.addEventListener("mouseOver", eventListener(obj, false));
        }

        for each (var obj: InteractiveObject in [guard]) {
            obj.addEventListener("focusIn", eventListener(obj, false));
            obj.addEventListener("focusOut", eventListener(obj, false));
            obj.addEventListener("mouseDown", eventListener(obj, false));
            obj.addEventListener("mouseUp", eventListener(obj, false));
            obj.addEventListener("click", eventListener(obj, false));
            obj.addEventListener("mouseFocusChange", eventListener(obj, false));
            obj.addEventListener("keyFocusChange", eventListener(obj, true));
            obj.addEventListener("rollOut", eventListener(obj, false));
            obj.addEventListener("rollOver", eventListener(obj, false));
            obj.addEventListener("mouseOver", eventListener(obj, false));
        }
    }

    private function objectToString(obj: DisplayObject): String {
        return "" + obj + ((obj != null) ? " (" + obj.name + ")" : "");
    }

    private function newSprite(): Sprite {
        var s:Sprite = new Sprite();
        s.graphics.beginFill(0x00FFFF);
        s.graphics.drawRect(0, 0, 100, 100);
        s.graphics.endFill();
        return s;
    }

    private function newMovieClip(buttonMode: Boolean, handCursor: Boolean): MovieClip {
        var mc:MovieClip = new MovieClip();
        mc.buttonMode = buttonMode;
        mc.useHandCursor = handCursor;
        mc.graphics.beginFill(0xFFCA00);
        mc.graphics.drawRect(0, 0, 100, 100);
        mc.graphics.endFill();
        return mc;
    }

    private function newTextField(): TextField {
        var tf:TextField = new TextField();
        tf.type = "input";
        tf.border = true;
        tf.width = 100;
        tf.height = 100;
        return tf;
    }

    private function newButton(): SimpleButton {
        var b:SimpleButton = new SimpleButton();
        b.downState = new ButtonDisplayState(0xFF0000, 100);
        b.overState = new ButtonDisplayState(0x0000FF, 100);
        b.upState = new ButtonDisplayState(0x000000, 100);
        b.hitTestState = new ButtonDisplayState(0, 100);
        b.useHandCursor  = true;
        return b;
    }
}
}
