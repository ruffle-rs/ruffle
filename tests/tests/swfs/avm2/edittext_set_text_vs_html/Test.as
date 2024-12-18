package {
import flash.display.*;
import flash.text.*;
import flash.events.*;
import flash.geom.*;

public class Test extends MovieClip {
    public function Test() {
        var tf = new TextField();
        tf.defaultTextFormat = new TextFormat("Default Font");
        tf.defaultTextFormat = new TextFormat("Default Font");

        tf.htmlText = "<font face='NonDefault Font'><b>bold</b></font>";

        trace("After setting HTML:");
        trace("  " + tf.text);
        trace("  " + tf.htmlText);

        tf.text = "text";

        trace("After setting text:");
        trace("  " + tf.text);
        trace("  " + tf.htmlText);

        tf.text = "<b>text</b>";

        trace("After setting text with HTML entities:");
        trace("  " + tf.text);
        trace("  " + tf.htmlText);
    }
}
}
