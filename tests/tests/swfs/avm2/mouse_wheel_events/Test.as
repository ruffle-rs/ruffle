package {
import flash.display.*;
import flash.events.MouseEvent;
import flash.text.*;

[SWF(width="300", height="200")]
public class Test extends MovieClip {
    [Embed(source="NotoSans.ttf", fontName="Noto Sans", embedAsCFF="false", unicodeRange="U+0020,U+0078,U+0030-U+0039")]
    private var notoSans:Class;

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
        tf.embedFonts = true;
        tf.multiline = true;
        tf.defaultTextFormat = new TextFormat("Noto Sans");
        tf.text = "xxxx 1\nxxxx 2\nxxxx 3\nxxxx 4\nxxxx 5\nxxxx 6\nxxxx 7\nxxxx 8\nxxxx 9\nxxxx 10\nxxxx 11\nxxxx 12\nxxxx 13\n"
        tf.mouseWheelEnabled = true;
        addChild(tf);
        return tf;
    }
}
}
