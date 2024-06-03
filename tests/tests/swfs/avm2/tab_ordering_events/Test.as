package {

import flash.display.MovieClip;
import flash.display.SimpleButton;
import flash.events.Event;
import flash.events.EventDispatcher;
import flash.events.KeyboardEvent;
import flash.text.TextField;

public class Test extends MovieClip {
    private var testStage:int = 0;
    private var clip:MovieClip;
    private var clip2:MovieClip;
    private var button:SimpleButton;
    private var button2:SimpleButton;
    private var text:TextField;

    public function Test() {
        super();

        var that:Test = this;
        stage.addEventListener("keyDown", function(evt:KeyboardEvent):void {
            if (evt.keyCode == 9) {
                trace("Tab pressed");
            }
            if (evt.keyCode == 27) {
                trace("Escape pressed");
                that.testStage += 1;
                trace("===== Stage " + that.testStage);
            }
        });

        clip = new MovieClip();
        clip.tabEnabled = true;
        clip.tabIndex = 1;
        setHandlers(clip, "clip");

        button = new SimpleButton();
        button.tabEnabled = true;
        button.tabIndex = 2;
        setHandlers(button, "button");

        text = new TextField();
        text.type = "input";
        text.tabEnabled = true;
        text.tabIndex = 3;
        setHandlers(text, "text");

        button2 = new SimpleButton();
        button2.tabEnabled = true;
        button2.tabIndex = 3;
        setHandlers(button2, "button2");

        clip2 = new MovieClip();
        clip2.tabEnabled = true;
        clip2.tabIndex = 4;
        setHandlers(clip2, "clip2");

        stage.addChild(clip);
        stage.addChild(clip2);
        stage.addChild(button);
        stage.addChild(button2);
        stage.addChild(text);

        trace("===== Setting focus manually");
        trace("Setting the focus to clip");
        stage.focus = clip;
        trace("Focus set to clip");
        trace("Setting the focus to button");
        stage.focus = button;
        trace("Focus set to button");
        trace("Setting the focus to text");
        stage.focus = text;
        trace("Focus set to text");
        trace("Setting the focus to button2");
        stage.focus = button2;
        trace("Focus set to button2");
        trace("Setting the focus to clip2");
        stage.focus = clip2;
        trace("Focus set to clip2");
        trace("===== Stage 0");
    }

    private function setHandlers(obj:EventDispatcher, name:String): void {
        var events:Array = [
            "keyDown",
            "keyUp",
            "click",
            "mouseDown",
            "mouseUp",
            "mouseOut",
            "mouseOver",
            "mouseMove",
            "rollOut",
            "rollOver",
            "focusIn",
            "focusOut",
            "keyFocusChange",
            "mouseFocusChange"
        ];
        var that:Test = this;
        for each (var event:String in events) {
            obj.addEventListener(event, function (evt:Event):void {
                if (evt instanceof KeyboardEvent) {
                    // ignore keyLocation, as it does not work properly at the time of writing this test
                    KeyboardEvent(evt).keyLocation = 0;
                }
                if (that.testStage == 2) {
                    return;
                }
                trace(name + "." + evt.type + ": " + evt.toString());
                if (that.testStage == 1) {
                    evt.preventDefault();
                }
            });
        }
    }
}
}
