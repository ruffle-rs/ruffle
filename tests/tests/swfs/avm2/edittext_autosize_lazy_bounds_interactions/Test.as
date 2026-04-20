package {
import flash.display.Sprite;
import flash.display.DisplayObject;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.events.Event;

public class Test extends Sprite {
    private var desc: String = null;

    public function Test() {
        stage.scaleMode = "noScale";

        testInteractions();

        trace("Tests finished");
    }

    private function testInteractions() {
        testBoundsUpdate("addChild", function(text:TextField, cb:Function) {
            addChild(text);
            cb();
        });

        testBoundsUpdate("contains", function(text:TextField, cb:Function) {
            contains(text);
            cb();
        });

        testBoundsUpdate("hitTestObject as argument", function(text:TextField, cb:Function) {
            hitTestObject(text);
            cb();
        });

        testBoundsUpdate("setting focus", function(text:TextField, cb:Function) {
            stage.focus = text;
            cb();
        });

        testBoundsUpdate("resetting focus", function(text:TextField, cb:Function) {
            stage.focus = text;
            cb();
        }, function(text:TextField) {
            stage.focus = text;
        });

        testBoundsUpdate("setting maskee", function(text:TextField, cb:Function) {
            var maskee = new Sprite();
            addChild(maskee);
            addChild(text);
            maskee.mask = text;
            cb();
        });

        testBoundsUpdate("getBounds relative", function(text:TextField, cb:Function) {
            var other = new Sprite();
            other.x = 0;
            other.y = 0;
            other.width = 10;
            other.height = 5;
            addChild(other);
            addChild(text);
            trace("// getBounds = " + other.getBounds(text));
            cb();
        });

        testBoundsUpdate("getRect relative", function(text:TextField, cb:Function) {
            var other = new Sprite();
            other.x = 0;
            other.y = 0;
            other.width = 10;
            other.height = 5;
            addChild(other);
            addChild(text);
            trace("// getRect = " + other.getRect(text));
            cb();
        });
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
