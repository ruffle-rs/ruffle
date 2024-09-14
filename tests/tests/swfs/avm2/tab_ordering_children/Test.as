package {

import flash.display.DisplayObject;
import flash.display.InteractiveObject;
import flash.display.Sprite;
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
    var clipOuter:Sprite;
    var text3:TextField;
    var text4:TextField;
    var clipInner:Sprite;
    var text5:TextField;
    var text6:TextField;

    var testStage:int = 0;

    public function Test() {
        super();

        this.text1 = this.newTextField(0);
        this.text1.name = "text1";
        this.text2 = this.newTextField(1);
        this.text2.name = "text2";
        this.text3 = this.newTextField(0);
        this.text3.name = "text3";
        this.text4 = this.newTextField(1);
        this.text4.name = "text4";
        this.text5 = this.newTextField(0);
        this.text5.name = "text5";
        this.text6 = this.newTextField(1);
        this.text6.name = "text6";

        this.clipInner = this.newMovieClip(2);
        this.clipInner.name = "clipInner";
        this.clipInner.addChild(this.text5);
        this.clipInner.addChild(this.text6);
        this.clipInner.width = 100;
        this.clipInner.height = 40;
        this.clipInner.y = 40;

        this.clipOuter = this.newMovieClip(2);
        this.clipOuter.name = "clipOuter";
        this.clipOuter.addChild(this.text3);
        this.clipOuter.addChild(this.text4);
        this.clipOuter.addChild(this.clipInner);
        this.clipOuter.width = 100;
        this.clipOuter.height = 80;
        this.clipOuter.y = 40;

        this.stage.addChild(this.text1);
        this.stage.addChild(this.text2);
        this.stage.addChild(this.clipOuter);

        var test:Test = this;
        this.stage.addEventListener("keyDown", function(evt:KeyboardEvent) {
            if (evt.keyCode == 27) {
                ++test.testStage;
                trace("===== Escape pressed, moving to stage " + test.testStage);
                test.setUpTestStage();
            } else if (evt.keyCode == 9) {
                trace("Tab pressed");
            }
        });

        this.stage.focus = this.text1;
    }

    function newTextField(i:int):TextField {
        var tf:TextField = new TextField();
        tf.type = "input";
        tf.border = true;
        tf.addEventListener("focusIn", function(obj) {
            return function (evt: FocusEvent): void {
                if (evt.relatedObject != null && evt.target != null) {
                    trace("Focus changed at " + obj.name + ": " + evt.relatedObject.name + " -> " + evt.target.name);
                }
            };
        }(tf));
        this.setupObject(tf, i);
        return tf;
    }

    function newMovieClip(i:int):Sprite {
        var mc:Sprite = new Sprite();
        mc.addEventListener("focusIn", function(obj) {
            return function (evt:FocusEvent):void {
                if (evt.relatedObject != null && evt.target != null) {
                    trace("Focus changed at " + obj.name + ": " + evt.relatedObject.name + " -> " + evt.target.name);
                }
            };
        }(mc));
        mc.height = 40;
        return mc;
    }

    function setupObject(o:DisplayObject, i:int):void {
        o.x = 0;
        o.y = i * 20;
        o.height = 20;
        o.width = 100;
    }

    function setUpTestStage():void {
        if (this.testStage == 0) {
            // already set up
        }
        if (this.testStage == 1) {
            this.clipOuter.tabChildren = true;
            this.clipInner.tabChildren = true;
        }
        if (this.testStage == 2) {
            this.clipOuter.tabChildren = false;
            this.clipInner.tabChildren = true;
        }
        if (this.testStage == 3) {
            this.clipOuter.tabChildren = true;
            this.clipInner.tabChildren = false;
        }
        if (this.testStage == 4) {
            this.clipOuter.tabChildren = false;
            this.clipOuter.tabEnabled = false;
            this.clipInner.tabChildren = true;
        }
        if (this.testStage == 5) {
            this.clipOuter.tabChildren = false;
            this.clipOuter.tabEnabled = true;
            this.clipInner.tabChildren = true;
        }
        if (this.testStage == 6) {
            this.clipOuter.tabChildren = false;
            this.clipOuter.tabEnabled = undefined;
            this.clipInner.tabChildren = true;

            this.text1.tabIndex = 3;
            this.text3.tabIndex = 1;
            this.text5.tabIndex = 2;
        }
        if (this.testStage == 7) {
            this.clipOuter.tabChildren = true;
            this.clipInner.tabChildren = false;

            this.text1.tabIndex = 3;
            this.text3.tabIndex = 1;
            this.text5.tabIndex = 2;
        }
    }
}
}
