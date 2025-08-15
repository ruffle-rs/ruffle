package {
import flash.display.InteractiveObject;
import flash.display.MovieClip;
import flash.display.Shape;
import flash.display.Sprite;
import flash.events.MouseEvent;
import flash.text.TextField;

[SWF(width="300", height="200")]
public class Test extends MovieClip {
    private var currentColor:int = 0;
    private var colors:Array = [
        0xFF0000,
        0x00FF00,
        0x0000FF
    ];

    private var text:TextField;
    private var sprite1:Sprite;
    private var sprite2:Sprite;
    private var shape:Shape;

    public function Test() {
        sprite1 = newSprite();
        sprite1.name = "sprite1";
        sprite2 = newSprite();
        sprite2.name = "sprite2";
        sprite2.x = 100;
        shape = newShape();
        shape.name = "shape";
        shape.x = 200;
        text = newText();
        text.name = "text";
        text.y = 100;

        function eventListener(obj: InteractiveObject, isText: Boolean): Function {
            return function(evt: MouseEvent): void {
                var str = evt.formatToString(
                            "MouseEvent", "type", "cancelable", "eventPhase",
                            "relatedObject", "ctrlKey", "altKey", "shiftKey");
                trace(obj.name + ", " + evt.target.name + ": " + str);
                if (isText) {
                    trace("  text scroll: " + text.scrollV);
                }
                evt.preventDefault();
            }
        }

        for each (var obj: InteractiveObject in [
            sprite1, sprite2, stage
        ]) {
            obj.addEventListener("mouseWheel", eventListener(obj, false));
        }
        text.addEventListener("mouseWheel", eventListener(text, true));

        stage.focus = sprite1;
    }

    private function newSprite(): Sprite {
        var s:Sprite = new Sprite();
        s.graphics.beginFill(0xFF00FF);
        s.graphics.drawRect(0, 0, 100, 100);
        s.graphics.endFill();
        addChild(s);
        return s;
    }

    private function newShape(): Shape {
        var s:Shape = new Shape();
        s.graphics.beginFill(colors[currentColor++]);
        s.graphics.drawRect(0, 0, 100, 100);
        s.graphics.endFill();
        addChild(s);
        return s;
    }

    private function newText(): TextField {
        var tf:TextField = new TextField();
        tf.multiline = true;
        tf.text = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13\n"
        tf.mouseWheelEnabled = true;
        addChild(tf);
        return tf;
    }
}
}
