package {
import flash.display.DisplayObject;
import flash.display.MovieClip;
import flash.display.Sprite;
import flash.events.MouseEvent;

[SWF(width="100", height="20")]
public class Test extends Sprite {
    private var colorIndex: int = 0;
    private var colors: Array = [
            0xFF0000,
            0x00FF00,
            0x0000FF,
            0x00FFFF,
            0xFF00FF,
            0xFFFF00
    ];
    private var events: Array = [
        MouseEvent.CLICK,
        MouseEvent.MOUSE_UP,
        MouseEvent.MOUSE_DOWN,
        MouseEvent.RIGHT_CLICK,
        MouseEvent.RIGHT_MOUSE_UP,
        MouseEvent.RIGHT_MOUSE_DOWN,
        MouseEvent.MIDDLE_CLICK,
        MouseEvent.MIDDLE_MOUSE_UP,
        MouseEvent.MIDDLE_MOUSE_DOWN
    ];

    public function Test() {
        var a = newMovieClip(0, 0, 20, 20, "A");
        var b = newMovieClip(20, 0, 40, 20, "B");
        b.addChild(newMovieClip(20, 0, 20, 20, "C"));
        var d = newMovieClip(60, 0, 20, 20, "D", false);
        addChild(a);
        addChild(b);
        addChild(d);
        for each (var eventName in [MouseEvent.MOUSE_UP, MouseEvent.RIGHT_MOUSE_UP, MouseEvent.MIDDLE_MOUSE_UP]) {
            registerListener(eventName, stage, "stage");
        }
    }

    private function newMovieClip(x:Number, y:Number, w:Number, h:Number, name:String, registerDownEvents:Boolean = true):MovieClip {
        var sprite:Sprite = new Sprite();
        sprite.graphics.beginFill(colors[colorIndex++]);
        sprite.graphics.drawRect(0, 0, w, h);
        sprite.tabEnabled = true;
        var movieClip:MovieClip = new MovieClip();
        movieClip.addChild(sprite);
        movieClip.x = x;
        movieClip.y = y;
        for each (var eventName in events) {
            if (!registerDownEvents && (
                eventName == MouseEvent.MOUSE_DOWN
                || eventName == MouseEvent.MIDDLE_MOUSE_DOWN
                || eventName == MouseEvent.RIGHT_MOUSE_DOWN
            )) {
                continue;
            }

            registerListener(eventName, sprite, name + ".sprite");
            registerListener(eventName, movieClip, name);
        }
        return movieClip;
    }

    private function registerListener(eventName:String, object:DisplayObject, name:String) {
        object.addEventListener(eventName, function(evt:MouseEvent):void {
            var formatted:String = evt.formatToString(
                    "MouseEvent","type","bubbles","cancelable",
                    "eventPhase","relatedObject","ctrlKey","altKey","shiftKey","buttonDown","delta");
            trace("Event " + eventName + " at " + name + ": " + formatted);
        });
    }
}
}
