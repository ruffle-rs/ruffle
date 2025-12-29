package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    public var child:TextField;

    public function Test() {
        trace("child.defaultTextFormat");
        printTextFormat(child.defaultTextFormat);
        trace("child.getTextFormat()");
        printTextFormat(child.getTextFormat());

        child.text = "x";

        trace("child.getTextFormat()");
        printTextFormat(child.getTextFormat());
        trace("child.getTextFormat(0,0)");
        try {
            printTextFormat(child.getTextFormat(0, 0));
        } catch (err:Error) {
            trace(err.getStackTrace());
        }
        trace("child.getTextFormat(0,1)");
        try {
            printTextFormat(child.getTextFormat(0, 1));
        } catch (err:Error) {
            trace(err.getStackTrace());
        }
        trace("child.getTextFormat(1,1)");
        try {
            printTextFormat(child.getTextFormat(1, 1));
        } catch (err:Error) {
            trace(err.getStackTrace());
        }
        trace("child.getTextFormat(1,0)");
        try {
            printTextFormat(child.getTextFormat(1, 0));
        } catch (err:Error) {
            trace(err.getStackTrace());
        }
        trace("child.getTextFormat(-1,0)");
        try {
            printTextFormat(child.getTextFormat(-1, 0));
        } catch (err:Error) {
            trace(err.getStackTrace());
        }
        trace("child.getTextFormat(0,-1)");
        try {
            printTextFormat(child.getTextFormat(0, -1));
        } catch (err:Error) {
            trace(err.getStackTrace());
        }

        child.text = "";

        trace("child.getTextFormat()");
        printTextFormat(child.getTextFormat());
    }

    private function printTextFormat(tf:TextFormat):void {
        trace("  align = " + tf.align + ";");
        trace("  blockIndent = " + tf.blockIndent + ";");
        trace("  bold = " + tf.bold + ";");
        trace("  bullet = " + tf.bullet + ";");
        trace("  color = " + tf.color + ";");
        trace("  font = " + tf.font + ";");
        trace("  indent = " + tf.indent + ";");
        trace("  italic = " + tf.italic + ";");
        trace("  kerning = " + tf.kerning + ";");
        trace("  leading = " + tf.leading + ";");
        trace("  leftMargin = " + tf.leftMargin + ";");
        trace("  letterSpacing = " + tf.letterSpacing + ";");
        trace("  rightMargin = " + tf.rightMargin + ";");
        trace("  size = " + tf.size + ";");
        trace("  tabStops = " + tf.tabStops + ";");
        trace("  target = " + tf.target + ";");
        trace("  underline = " + tf.underline + ";");
        trace("  url = " + tf.url + ";");
    }
}
}
