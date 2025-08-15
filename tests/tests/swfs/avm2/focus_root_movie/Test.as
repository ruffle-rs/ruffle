package {

import flash.display.MovieClip;
import flash.display.Shape;
import flash.events.FocusEvent;

[SWF(width="20", height="20", backgroundColor="#000000")]
public class Test extends MovieClip {
    public function Test() {
        super();
        addChild(newShape(0));

        var root: MovieClip = MovieClip(this.root);
        root.tabEnabled = true;
        root.tabIndex = 1;
        root.focusRect = true;
        root.buttonMode = true;

        var clip: MovieClip = new MovieClip();
        clip.tabEnabled = true;
        clip.tabIndex = 2;
        clip.focusRect = true;
        clip.addChild(newShape(1));
        stage.addChild(clip);

        stage.addEventListener("focusIn", function(obj) {
            return function (evt: FocusEvent): void {
                if (evt.relatedObject != null && evt.target != null) {
                    trace("Focus changed at " + obj.name + ": " + evt.relatedObject.name + " -> " + evt.target.name);
                }
            };
        }(stage));

        stage.focus = clip;
        trace("After set: " + stage.focus);
        stage.focus = root;
        trace("After set: " + stage.focus);
        trace("=====");
    }

    function newShape(n: int): Shape {
        var shape = new Shape();
        shape.graphics.beginFill(0xFF0000);
        shape.graphics.drawRect(0, 0, 10, 10);
        shape.graphics.endFill();
        shape.x = 10 * n;
        shape.y = 10 * n;
        return shape;
    }
}
}
