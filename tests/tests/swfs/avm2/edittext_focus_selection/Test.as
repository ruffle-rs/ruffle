package {
import flash.display.*;
import flash.text.*;
import flash.events.*;

public class Test extends MovieClip {
    public function Test() {
        var tf = new TextField();
        tf.type = "input";
        addChild(tf);

        tf.text = "abcd";
        trace(tf.selectionBeginIndex + "," + tf.selectionEndIndex);

        stage.focus = tf;

        trace(tf.selectionBeginIndex + "," + tf.selectionEndIndex);

        stage.focus = null;

        trace(tf.selectionBeginIndex + "," + tf.selectionEndIndex);

        tf.setSelection(1, 2);
        trace(tf.selectionBeginIndex + "," + tf.selectionEndIndex);

        stage.focus = tf;

        trace(tf.selectionBeginIndex + "," + tf.selectionEndIndex);
    }
}
}
