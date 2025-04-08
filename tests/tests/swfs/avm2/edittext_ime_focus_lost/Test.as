package {
import flash.display.*;
import flash.text.*;
import flash.events.*;
import flash.utils.setTimeout;

[SWF(width="100", height="100", frameRate="1")]
public class Test extends MovieClip {
    public function Test() {
        var tf = new TextField();
        tf.border = true;
        tf.type = "input";
        tf.width = 100;
        tf.height = 100;
        addChild(tf);
        stage.focus = tf;

        setTimeout(function() {
            trace("Before: " + tf.text);
            trace("  " + tf.length);
            trace("  " + tf.selectionBeginIndex);
            trace("  " + tf.selectionEndIndex);

            stage.focus = null;

            trace("After: " + tf.text);
            trace("  " + tf.length);
            trace("  " + tf.selectionBeginIndex);
            trace("  " + tf.selectionEndIndex);
        }, 4500);

        tf.addEventListener("textInput", function(evt:TextEvent):void {
            trace("input " + evt.text);
        });
    }
}
}
