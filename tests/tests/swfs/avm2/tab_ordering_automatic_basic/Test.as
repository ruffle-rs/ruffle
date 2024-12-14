package {

import flash.display.DisplayObject;
import flash.display.InteractiveObject;
import flash.events.KeyboardEvent;
import flash.text.TextField;
import flash.display.Sprite;
import flash.display.SimpleButton;
import flash.display.MovieClip;
import flash.events.Event;
import flash.events.FocusEvent;

public class Test extends MovieClip {
    var text1:TextField;
    var text2:TextField;
    var text3:TextField;
    var text4:TextField;
    var testStage:int = 0;

    public function Test() {
        super();

        text1 = this.newTextField(1);
        text2 = this.newTextField(2);
        text3 = this.newTextField(3);
        text4 = this.newTextField(4);

        this.stage.focus = text1;

        var test:Test = this;
        for each (var obj in [text1, text2, text3, text4]) {
            obj.addEventListener("focusIn", function (evt:FocusEvent):void {
                trace("Focus changed: " + evt.relatedObject.name + " -> " + evt.target.name);
            });
            this.stage.addChild(obj);
        }

        this.stage.addEventListener("keyDown", function(evt:KeyboardEvent) {
            if (evt.keyCode == 27) {
                test.testStage += 1;
                trace("Escape pressed, moving to stage " + test.testStage);
                test.setUpTestStage();
            } else if (evt.keyCode == 9) {
                trace("Tab pressed");
            }
        });
    }

    function setUpTestStage() {
        if (this.testStage == 0) {
            // already set up
        }
        if (this.testStage == 1) {
            this.stage.focus = text4;
        }
        if (this.testStage == 2) {
            this.stage.focus = text1;
            text2.tabEnabled = false;
        }
        if (this.testStage == 3) {
            this.stage.focus = text1;
            text1.tabEnabled = false;
            text2.tabEnabled = true;
        }
    }

    function newTextField(i:int):TextField {
        var tf:TextField = new TextField();
        tf.type = "input";
        tf.name = "text" + i;
        tf.border = true;
        tf.x = 0;
        tf.y = i * 20;
        tf.height = 20;
        tf.width = 100;
        return tf;
    }
}
}
