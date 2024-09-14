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

public class Test extends MovieClip {
    public function Test() {
        super();

        var text1:TextField = new TextField();
        var text2:TextField = new TextField();
        text2.type = "input";
        var button:SimpleButton = new SimpleButton();
        var mc1:MovieClip = new MovieClip();
        var mc2:MovieClip = new MovieClip();
        mc2.buttonMode = true;
        var sprite:Sprite = new Sprite();

        trace("===== stage =====");
        this.testProperty(this.stage);
        trace("===== text1 =====");
        this.testProperty(text1);
        trace("===== text2 =====");
        this.testProperty(text2);
        trace("===== button =====");
        this.testProperty(button);
        trace("===== mc1 =====");
        this.testProperty(mc1);
        trace("===== mc2 =====");
        this.testProperty(mc2);
        trace("===== sprite =====");
        this.testProperty(sprite);
    }

    function logError(f:*):void {
        try {
            f();
        } catch (error:Error) {
            trace('    Error: ' + error);
        }
    }

    function testProperty(obj:InteractiveObject):void {
        trace("  default value: " + obj.focusRect);
        this.testPropertyValue(obj, true);
        this.testPropertyValue(obj, false);
        this.testPropertyValue(obj, null);
        this.testPropertyValue(obj, undefined);
        this.testPropertyValue(obj, 0);
        this.testPropertyValue(obj, 1);
        this.testPropertyValue(obj, -1);
        this.testPropertyValue(obj, 0.2);
        this.testPropertyValue(obj, 'test');
        this.testPropertyValue(obj, 1.0/0.0);
        this.testPropertyValue(obj, 0.0/0.0);
        this.testPropertyValue(obj, new Object());
    }

    function testPropertyValue(obj:InteractiveObject, value:*):void {
        this.logError(function() {
            obj.focusRect = value;
        });
        trace("  after set to " + value + ": " + obj.focusRect);

    }
}
}
