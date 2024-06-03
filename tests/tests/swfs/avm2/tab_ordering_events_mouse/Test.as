package {

import flash.display.MovieClip;
import flash.display.SimpleButton;
import flash.events.Event;
import flash.events.EventDispatcher;
import flash.events.KeyboardEvent;

[SWF(width='200', height='600')]
public class Test extends MovieClip {
    private var button:SimpleButton;
    private var button2:SimpleButton;
    private var logEnabled:Boolean = false;

    public function Test() {
        super();

        var that:Test = this;
        stage.addEventListener("keyDown", function(evt:KeyboardEvent):void {
            if (evt.keyCode == 9 && that.logEnabled) {
                trace("Tab pressed");
            }
            if (evt.keyCode == 27) {
                stage.focus = button;
                trace("Escape pressed");
                that.logEnabled = true;
            }
        });

        button = newButton();
        button.x = 0;
        button.y = 200;
        button.tabEnabled = true;
        button.tabIndex = 1;

        button2 = newButton();
        button2.x = 0;
        button2.y = 400;
        button2.tabEnabled = true;
        button2.tabIndex = 2;

        stage.addChild(button);
        stage.addChild(button2);
        stage.focus = button;

        setHandlers(button, "button");
        setHandlers(button2, "button2");
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
                if (that.logEnabled) {
                    trace(name + "." + evt.type + ": " + evt.toString());
                }
            })
        }
    }

    private function newButton(): SimpleButton {
        var b:SimpleButton = new SimpleButton();
        b.downState = new ButtonDisplayState(0xFF0000, 200);
        b.overState = new ButtonDisplayState(0x0000FF, 200);
        b.upState = new ButtonDisplayState(0x000000, 200);
        b.hitTestState = new ButtonDisplayState(0, 200);
        b.useHandCursor  = true;
        return b;
    }
}
}
