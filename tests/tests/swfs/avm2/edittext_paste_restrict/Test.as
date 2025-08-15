package {
import flash.display.*;
import flash.text.*;
import flash.events.*;

public class Test extends MovieClip {
    public function Test() {
        var tf = new TextField();
        tf.type = "input";
        tf.border = true;
        tf.x = 10;
        tf.y = 10;
        tf.width = 200;
        tf.height = 50;
        tf.restrict = "y";
        tf.text = "xxxxx";
        addChild(tf);

        tf.addEventListener("textInput", function(evt:TextEvent):void {
            trace("input " + evt.text + ", " + tf.text);
        });

        tf.addEventListener("change", function(evt:Event):void {
            trace("change " + tf.text);
        });

        stage.addEventListener("keyDown", function(evt:KeyboardEvent):void {
            if (evt.keyCode >= 0 && evt.keyCode <= 256) {
                trace("key down " + evt.keyCode + ", " + tf.text);
            }
        });

        stage.addEventListener("keyUp", function(evt:KeyboardEvent):void {
            if (evt.keyCode >= 0 && evt.keyCode <= 256) {
                trace("key up " + evt.keyCode + ", " + tf.text);
            }
        });

        stage.focus = tf;
        tf.setSelection(3, 5);
    }
}
}
