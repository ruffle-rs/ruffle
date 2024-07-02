package {

import flash.display.InteractiveObject;
import flash.display.MovieClip;
import flash.display.SimpleButton;
import flash.display.Sprite;
import flash.events.Event;
import flash.events.KeyboardEvent;
import flash.events.MouseEvent;
import flash.text.TextField;
import flash.utils.getQualifiedClassName;

[SWF(width="600", height="200")]
public class Test extends MovieClip {
    private var spriteA: Sprite;
    private var mc1A: MovieClip;
    private var mc2A: MovieClip;
    private var mc3A: MovieClip;
    private var textA: TextField;
    private var buttonA: SimpleButton;
//    private var spriteB: Sprite;
//    private var mc1B: MovieClip;
//    private var mc2B: MovieClip;
//    private var mc3B: MovieClip;
//    private var textB: TextField;
//    private var buttonB: SimpleButton;
//    private var spriteC: Sprite;
//    private var mc1C: MovieClip;
//    private var mc2C: MovieClip;
//    private var mc3C: MovieClip;
//    private var textC: TextField;
//    private var buttonC: SimpleButton;

    public function Test() {
        super();

        spriteA = newSprite();
        spriteA.name = "spriteA";
        spriteA.x = 0;
        spriteA.y = 100;
        mc1A = newMovieClip(false, true);
        mc1A.name = "mc1A";
        mc1A.x = 100;
        mc1A.y = 100;
        mc2A = newMovieClip(true, true);
        mc2A.name = "mc2A";
        mc2A.x = 200;
        mc2A.y = 100;
        mc3A = newMovieClip(true, false);
        mc3A.name = "mc3A";
        mc3A.x = 300;
        mc3A.y = 100;
        textA = newTextField();
        textA.name = "textA";
        textA.x = 400;
        textA.y = 100;
        buttonA = newButton();
        buttonA.name = "buttonA";
        buttonA.x = 500;
        buttonA.y = 100;

//        spriteB = newSprite();
//        spriteB.name = "spriteB";
//        spriteB.x = 0;
//        spriteB.y = 200;
//        mc1B = newMovieClip(false, true);
//        mc1B.name = "mc1B";
//        mc1B.x = 100;
//        mc1B.y = 200;
//        mc2B = newMovieClip(true, true);
//        mc2B.name = "mc2B";
//        mc2B.x = 200;
//        mc2B.y = 200;
//        mc3B = newMovieClip(true, false);
//        mc3B.name = "mc3B";
//        mc3B.x = 300;
//        mc3B.y = 200;
//        textB = newTextField();
//        textB.name = "textB";
//        textB.x = 400;
//        textB.y = 200;
//        buttonB = newButton();
//        buttonB.name = "buttonB";
//        buttonB.x = 500;
//        buttonB.y = 200;
//
//        spriteC = newSprite();
//        spriteC.name = "spriteC";
//        spriteC.x = 0;
//        spriteC.y = 300;
//        mc1C = newMovieClip(false, true);
//        mc1C.name = "mc1C";
//        mc1C.x = 100;
//        mc1C.y = 300;
//        mc2C = newMovieClip(true, true);
//        mc2C.name = "mc2C";
//        mc2C.x = 200;
//        mc2C.y = 300;
//        mc3C = newMovieClip(true, false);
//        mc3C.name = "mc3C";
//        mc3C.x = 300;
//        mc3C.y = 300;
//        textC = newTextField();
//        textC.name = "textC";
//        textC.x = 400;
//        textC.y = 300;
//        buttonC = newButton();
//        buttonC.name = "buttonC";
//        buttonC.x = 500;
//        buttonC.y = 300;

        spriteA.tabEnabled = true;
        spriteA.tabIndex = 1;
        mc1A.tabEnabled = true;
        mc1A.tabIndex = 2;
        mc2A.tabEnabled = true;
        mc2A.tabIndex = 3;
        mc3A.tabEnabled = true;
        mc3A.tabIndex = 4;
        textA.tabEnabled = true;
        textA.tabIndex = 5;
        buttonA.tabEnabled = true;
        buttonA.tabIndex = 6;

//        spriteB.tabEnabled = true;
//        spriteB.tabIndex = 7;
//        mc1B.tabEnabled = true;
//        mc1B.tabIndex = 8;
//        mc2B.tabEnabled = true;
//        mc2B.tabIndex = 9;
//        mc3B.tabEnabled = true;
//        mc3B.tabIndex = 10;
//        textB.tabEnabled = true;
//        textB.tabIndex = 11;
//        buttonB.tabEnabled = true;
//        buttonB.tabIndex = 12;
//
//        spriteC.tabEnabled = true;
//        spriteC.tabIndex = 13;
//        mc1C.tabEnabled = true;
//        mc1C.tabIndex = 14;
//        mc2C.tabEnabled = true;
//        mc2C.tabIndex = 15;
//        mc3C.tabEnabled = true;
//        mc3C.tabIndex = 16;
//        textC.tabEnabled = true;
//        textC.tabIndex = 17;
//        buttonC.tabEnabled = true;
//        buttonC.tabIndex = 18;

        stage.addChild(spriteA);
        stage.addChild(mc1A);
        stage.addChild(mc2A);
        stage.addChild(mc3A);
        stage.addChild(textA);
        stage.addChild(buttonA);
//        stage.addChild(spriteB);
//        stage.addChild(mc1B);
//        stage.addChild(mc2B);
//        stage.addChild(mc3B);
//        stage.addChild(textB);
//        stage.addChild(buttonB);
//        stage.addChild(spriteC);
//        stage.addChild(mc1C);
//        stage.addChild(mc2C);
//        stage.addChild(mc3C);
//        stage.addChild(textC);
//        stage.addChild(buttonC);

        stage.addEventListener("keyDown", function(evt:KeyboardEvent):void {
            if (evt.keyCode == 27) {
                trace("==================== Escape pressed");
            } else if (evt.keyCode == 9) {
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
                trace(obj.name + ", " + evt.target.name + ": " + str);
            }
        }

        for each (var obj: InteractiveObject in [
            spriteA, mc1A, mc2A, mc3A, textA, buttonA,
//            spriteB, mc1B, mc2B, mc3B, textB, buttonB,
//            spriteC, mc1C, mc2C, mc3C, textC, buttonC,
            stage
        ]) {
            obj.addEventListener("focusIn", eventListener(obj));
            obj.addEventListener("focusOut", eventListener(obj));
            obj.addEventListener("mouseDown", eventListener(obj));
            obj.addEventListener("mouseUp", eventListener(obj));
            obj.addEventListener("click", eventListener(obj));
            obj.addEventListener("mouseFocusChange", eventListener(obj));
            obj.addEventListener("keyFocusChange", eventListener(obj));
        }

//        for each (var obj: InteractiveObject in [
//            spriteB, mc1B, mc2B, mc3B, textB, buttonB
//        ]) {
//            obj.addEventListener("mouseFocusChange", eventListener(obj));
//        }
//
//        for each (var obj: InteractiveObject in [
//            spriteC, mc1C, mc2C, mc3C, textC, buttonC
//        ]) {
//            obj.addEventListener("keyFocusChange", eventListener(obj));
//        }
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
        if (buttonMode) {
            if (handCursor) {
                mc.graphics.beginFill(0xFFCA00);
            } else {
                mc.graphics.beginFill(0xCAFF00);
            }
        } else {
            mc.graphics.beginFill(0x00FF00);
        }
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
