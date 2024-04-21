package {

import flash.display.DisplayObject;
import flash.display.InteractiveObject;
import flash.display.InteractiveObject;
import flash.display.Sprite;
import flash.events.KeyboardEvent;
import flash.text.TextField;
import flash.display.Sprite;
import flash.display.SimpleButton;
import flash.display.MovieClip;
import flash.events.Event;
import flash.events.FocusEvent;

[SWF(width="50", height="50", backgroundColor="#000000")]
public class Test extends MovieClip {
    var clip1:Sprite;
    var clip2:Sprite;
    var testStage:int = 0;
    var logFocus:Boolean = false;

    public function Test() {
        super();

        this.clip1 = new Sprite();
        this.clip1.x = 10;
        this.clip1.y = 10;
        this.clip1.graphics.beginFill(0xFF66CC);
        this.clip1.graphics.drawRect(0, 0, 20, 20);

        this.clip2 = new Sprite();
        this.clip2.x = 20;
        this.clip2.y = 20;
        this.clip2.graphics.beginFill(0xFF66CC);
        this.clip2.graphics.drawRect(0, 0, 20, 20);

        this.clip1.name = "clip1";
        this.clip1.tabEnabled = true;
        this.clip1.tabIndex = 1;
        this.clip2.name = "clip2";
        this.clip2.tabEnabled = true;
        this.clip2.tabIndex = 2;

        this.stage.addChild(this.clip1);
        this.stage.addChild(this.clip2);

        var test:Test = this;
        this.clip1.addEventListener("focusIn", function (evt:FocusEvent):void {
            if (test.logFocus && evt.relatedObject != null && evt.target != null) {
                trace("Focus changed: " + evt.relatedObject.name + " -> " + evt.target.name);
            }
        });
        this.clip2.addEventListener("focusIn", function (evt:FocusEvent):void {
            if (test.logFocus && evt.relatedObject != null && evt.target != null) {
                trace("Focus changed: " + evt.relatedObject.name + " -> " + evt.target.name);
            }
        });
        this.stage.addEventListener("keyDown", function(evt:KeyboardEvent) {
            if (evt.keyCode == 27) {
                test.nextTestStage();
            }
        });
    }

    function nextTestStage() {
        this.testStage += 1;
        trace("Setting test stage to " + this.testStage);
        if (testStage == 1) {
            this.stage.stageFocusRect = true;
        } else if (testStage == 2) {
            this.stage.stageFocusRect = false;
        } else if (testStage == 3) {
            this.stage.stageFocusRect = true;
            this.clip1.focusRect = false;
        } else if (testStage == 4) {
            this.stage.stageFocusRect = false;
            this.clip1.focusRect = true;
        } else if (testStage == 5) {
            this.stage.stageFocusRect = true;
            this.clip1.focusRect = null;
        } else if (testStage == 6) {
            this.stage.stageFocusRect = false;
            this.clip1.focusRect = null;
        }
        this.stage.focus = this.clip2;
        this.logFocus = true;
    }
}
}
