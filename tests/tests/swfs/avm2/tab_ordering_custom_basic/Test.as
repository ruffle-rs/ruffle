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
    var text5:TextField;
    var text6:TextField;

    public function Test() {
        super();

        text1 = this.newTextField(1);
        text2 = this.newTextField(2);
        text3 = this.newTextField(3);
        text4 = this.newTextField(4);
        text5 = this.newTextField(5);
        text6 = this.newTextField(6);

        text2.tabIndex = 2;
        text3.tabIndex = 1;
        text4.tabIndex = 4;
        text4.tabEnabled = false;
        text5.tabIndex = 3;
        text6.tabIndex = 5;

        this.stage.focus = text1;

        var test:Test = this;
        for each (var obj in [text1, text2, text3, text4, text5, text6]) {
            obj.addEventListener("focusIn", function (evt:FocusEvent):void {
                trace("Focus changed: " + evt.relatedObject.name + " -> " + evt.target.name);
            });
            this.stage.addChild(obj);
        }

        this.stage.addEventListener("keyDown", function(evt:KeyboardEvent) {
            if (evt.keyCode == 27) {
                trace("Escape pressed");
                test.stage.focus = test.text5;
            } else if (evt.keyCode == 9) {
                trace("Tab pressed");
            }
        });
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
