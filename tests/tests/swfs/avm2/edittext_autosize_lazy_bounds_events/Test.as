package {
import flash.display.Sprite;
import flash.display.DisplayObject;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.events.Event;
import flash.utils.setTimeout;

public class Test extends Sprite {
    private var desc: String = null;
    private var cb: Function = null;
    private var currentFrame: int = 0;

    public function Test() {
        stage.scaleMode = "noScale";

        testRender();
        testEvents();

        trace("Tests finished");
    }

    private function testRender() {
        testBoundsUpdate("none", function(text:TextField, cb:Function) {
            cb();
        });

        // NOTE: setTimeout with 0 time produces nondeterministic
        //   results, make sure its > than frame time

        testBoundsUpdate("set timeout", function(text:TextField, cb:Function) {
            setTimeout(cb, 100);
        });
        testBoundsUpdate("add child + set timeout", function(text:TextField, cb:Function) {
            addChild(text);
            setTimeout(cb, 100);
        });

        var that = this;
        setTimeout(function():void {
            that.cb();
        }, 100);
        testBoundsUpdate("add child + set timeout before construction", function(text:TextField, cb:Function) {
            addChild(text);
            that.cb = cb;
        });

        testBoundsUpdate("enter frame", function(text:TextField, cb:Function) {
            runEventOnce(cb, "enterFrame");
        });
        testBoundsUpdate("frame constructed", function(text:TextField, cb:Function) {
            runEventOnce(cb, "frameConstructed");
        });
        testBoundsUpdate("exit frame", function(text:TextField, cb:Function) {
            runEventOnce(cb, "exitFrame");
        });

        testBoundsUpdate("add child", function(text:TextField, cb:Function) {
            addChild(text);
            cb();
        });
        testBoundsUpdate("add child + enter frame", function(text:TextField, cb:Function) {
            addChild(text);
            runEventOnce(cb, "enterFrame");
        });
        testBoundsUpdate("add child + frame constructed", function(text:TextField, cb:Function) {
            addChild(text);
            runEventOnce(cb, "frameConstructed");
        });
        testBoundsUpdate("add child + exit frame", function(text:TextField, cb:Function) {
            addChild(text);
            runEventOnce(cb, "exitFrame");
        });

        testBoundsUpdate("invisible + add child + enter frame", function(text:TextField, cb:Function) {
            text.visible = "false";
            addChild(text);
            cb();
        });
        testBoundsUpdate("add child + invisible + enter frame", function(text:TextField, cb:Function) {
            addChild(text);
            text.visible = "false";
            cb();
        });
        testBoundsUpdate("add child + remove child + enter frame", function(text:TextField, cb:Function) {
            addChild(text);
            removeChild(text);
            runEventOnce(cb);
        });
    }

    private function testEvents() {
        var test = this;
        var events = ["enterFrame", "frameConstructed", "exitFrame"];

        for each (var fromEventName in events) {
            runEventOnce(function(): void {
                var fromFrame = test.currentFrame;
                for each (var toEventName in events) {
                    testBoundsUpdate("event " + fromEventName + " -> " + toEventName, function(text:TextField, cb:Function) {
                        addChild(text);
                        runEventOnce(function():void {
                            var toFrame = test.currentFrame;
                            trace("// " + "event " + fromEventName + " -> " + toEventName);
                            trace("// " + fromFrame + " -> " + toFrame);
                            cb();
                        }, toEventName);
                    });
                }
            }, fromEventName);
        }
    }

    private function runEventOnce(fun: Function, eventName: String = Event.ENTER_FRAME, target: DisplayObject = null):void {
        if (target == null) {
            target = stage;
        }
        function handler(event:Event):void {
            target.removeEventListener(eventName, handler);
            fun();
        }

        target.addEventListener(eventName, handler);
    }

    private function testBoundsUpdate(desc: String, fun: Function, before: Function = null):void {
        this.desc = desc;
        var text = new TextField();
        text.width = 100;
        text.height = 20;
        if (before != null) {
            before(text);
        }

        text.autoSize = "center";
        fun(text, function():void {
            text.wordWrap = true;
            trace("Testing: " + desc);
            trace("  " + text.x + "," + text.y + "," + text.width + "," + text.height);
        });
    }
}
}
