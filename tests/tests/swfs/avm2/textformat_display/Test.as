package {
import flash.display.*;
import flash.text.*;

public class Test extends Sprite {
    public function Test() {
        var tf = new TextFormat();
        trace(tf.display);
        tf.display = "none";
        trace(tf.display);
        tf.display = "inline";
        trace(tf.display);
        tf.display = "block";
        trace(tf.display);
        tf.display = "unknown";
        trace(tf.display);
        tf.display = "inline";
        trace(tf.display);

        try {
            tf.display = null;
        } catch (e) {
            trace(e.getStackTrace());
        }
        trace(tf.display);

        try {
            tf.display = undefined;
        } catch (e) {
            trace(e.getStackTrace());
        }
        trace(tf.display);
    }
}
}
