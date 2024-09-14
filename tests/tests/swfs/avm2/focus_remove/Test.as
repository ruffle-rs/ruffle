package {

import flash.display.DisplayObject;
import flash.display.InteractiveObject;
import flash.display.InteractiveObject;
import flash.display.InteractiveObject;
import flash.display.SimpleButton;
import flash.display.Sprite;
import flash.events.KeyboardEvent;
import flash.text.TextField;
import flash.text.TextField;
import flash.display.Sprite;
import flash.display.SimpleButton;
import flash.display.MovieClip;
import flash.events.Event;
import flash.events.FocusEvent;

[SWF(width="50", height="50", backgroundColor="#000000")]
public class Test extends MovieClip {
    public function Test() {
        super();

        trace("===== sprite");
        testObject(new Sprite());
        trace("===== clip");
        testObject(new MovieClip());
        trace("===== text");
        testObject(new TextField());
        trace("===== button");
        testObject(new SimpleButton());
    }

    function testObject(obj:InteractiveObject) {
        stage.focus = obj;
        trace("Focus: " + stage.focus);
        stage.addChild(obj);
        trace("Focus: " + stage.focus);
        stage.removeChild(obj);
        trace("Focus: " + stage.focus);
        stage.focus = obj;
        trace("Focus: " + stage.focus);
    }
}
}
