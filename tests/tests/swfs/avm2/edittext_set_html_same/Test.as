package {
import flash.display.*;
import flash.text.*;
import flash.events.*;
import flash.geom.*;

public class Test extends MovieClip {
    public function Test() {
        var tf = new TextField();
        tf.multiline = true;
        tf.defaultTextFormat = new TextFormat("Unknown Font");
        tf.htmlText = "<p align=\"center\">x</p>";

        trace("Before newline removal:");
        trace("  " + tf.text.length);

        tf.replaceText(1, 2, "");

        trace("After newline removal:");
        trace("  " + tf.text.length);
        trace("  " + tf.htmlText);

        tf.htmlText = "<P ALIGN=\"CENTER\"><FONT FACE=\"Unknown Font\" SIZE=\"12\" COLOR=\"#000000\" LETTERSPACING=\"0\" KERNING=\"0\">x</FONT></P>";

        trace("After setting to the same value:");
        trace("  " + tf.text.length);
        trace("  " + tf.htmlText);

        tf.htmlText = "<P  ALIGN=\"CENTER\"><FONT FACE=\"Unknown Font\" SIZE=\"12\" COLOR=\"#000000\" LETTERSPACING=\"0\" KERNING=\"0\">x</FONT></P>";

        trace("After setting to a slightly different value:");
        trace("  " + tf.text.length);
        trace("  " + tf.htmlText);

        tf.htmlText = "<font face='Unknown Font 2'><b>x</b></font>";

        trace("After setting to HTML x:");
        trace("  " + tf.text.length);
        trace("  " + tf.htmlText);

        tf.text = "x";

        trace("After setting text to x:");
        trace("  " + tf.text.length);
        trace("  " + tf.htmlText);
    }
}
}
