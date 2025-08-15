package {

import flash.display.DisplayObject;
import flash.display.InteractiveObject;
import flash.events.KeyboardEvent;
import flash.text.TextField;
import flash.display.Sprite;
import flash.display.SimpleButton;
import flash.display.MovieClip;
import flash.events.Event;

public class Test extends MovieClip {
    var objects:Array;
    var tabbedObjects:Array = [];
    var testStage:int = 0;

    public function Test() {
        super();

        var text1:TextField = this.newTextField(1);
        text1.type = "dynamic";

        var text2:TextField = this.newTextField(2);
        text2.maxChars = 0;

        var text3:TextField = this.newTextField(3);
        text3.visible = false;

        var text4:TextField = this.newTextField(4);
        text4.x = -400;
        text4.y = -400;

        var text5:TextField = this.newTextField(5);

        var text6:TextField = this.newTextField(6);
        text6.selectable = false;

        var text7:TextField = this.newTextField(7);
        text7.width = 0;
        text7.height = 0;

        var clip8:MovieClip = this.newMovieClip(8);

        var clip9:MovieClip = this.newMovieClip(9);
        clip9.visible = false;

        var button10:SimpleButton = new SimpleButton();
        button10.name = "button10";
        setupObject(button10, 10);

        var button11:SimpleButton = new SimpleButton();
        button11.name = "button11";
        setupObject(button11, 11);
        button11.visible = false;

        var text12:TextField = this.newTextField(12);
        text12.tabEnabled = true;
        text12.type = "dynamic";

        this.objects = [
            text1,
            text2,
            text3,
            text4,
            text5,
            text6,
            text7,
            clip8,
            clip8.getChildByName("clip8.text"),
            clip9,
            clip9.getChildByName("clip9.text"),
            button10,
            button11,
            text12
        ];

        var test:Test = this;
        for each (var obj in objects) {
            obj.addEventListener("focusIn", function(obj) {
                return function (evt:Event):void {
                    test.tabbedObjects.push(obj.name);
                }
            }(obj));
            this.stage.addChild(obj);
        }

        this.stage.addEventListener("keyDown", function(evt:KeyboardEvent) {
            if (evt.keyCode == 27) {
                trace("Tabbable elements:");
                for each (var obj in objects) {
                    var exists = false;
                    for each (var name in tabbedObjects) {
                        if (obj.name == name) {
                            exists = true;
                        }
                    }
                    trace("  " + obj.name + ": " + exists);
                }

                ++test.testStage;
                if (test.testStage == 1) {
                    trace("Enabling tab");
                    for each (var obj in objects) {
                        obj.tabEnabled = true;
                    }
                } else if (test.testStage == 2) {
                    trace("Setting custom order");
                    for (var i in objects) {
                        objects[i].tabIndex = i;
                    }
                }
            }
        });
    }

    function newTextField(i:int):TextField {
        var tf:TextField = new TextField();
        tf.type = "input";
        tf.name = "text" + i;
        tf.border = true;
        this.setupObject(tf, i);
        return tf;
    }

    function newMovieClip(i:int):MovieClip {
        var mc:MovieClip = new MovieClip();
        mc.name = "clip" + i;
        this.setupObject(mc, i);

        var tf:TextField = this.newTextField(0);
        tf.name = mc.name + ".text";

        mc.addChild(tf);
        return mc;
    }

    function setupObject(o:DisplayObject, i:int):void {
        o.x = 0;
        o.y = i * 20;
        o.height = 20;
        o.width = 100;
    }
}
}
