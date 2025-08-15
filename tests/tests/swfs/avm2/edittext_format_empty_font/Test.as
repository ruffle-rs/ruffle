package {
import flash.display.MovieClip;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends MovieClip {
    public function Test() {
        var tf = new TextField();
        tf.defaultTextFormat = new TextFormat("x");
        trace("after setting to x: " + tf.defaultTextFormat.font);
        tf.defaultTextFormat = new TextFormat("");
        trace("after setting to empty: " + tf.defaultTextFormat.font);
        tf.defaultTextFormat = new TextFormat(null);
        trace("after setting to null: " + tf.defaultTextFormat.font);

        trace("null font in format: " + new TextFormat(null).font);
        trace("empty font in format: " + new TextFormat("").font);

        tf.text = "text";
        trace("after setting to x: " + tf.getTextFormat(0, 1).font);
        tf.setTextFormat(new TextFormat(""), 0, 1);
        trace("after setting to empty: " + tf.getTextFormat(0, 1).font);
        tf.setTextFormat(new TextFormat(null), 0, 1);
        trace("after setting to null: " + tf.getTextFormat(0, 1).font);
    }
}
}
