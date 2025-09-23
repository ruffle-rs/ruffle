package {
import flash.display.*;
import flash.text.*;

public class Test extends MovieClip {
    public function Test() {
        // Whitespace at the end
        testCss(".s { color: #ff0000 ;}");
        testCss(".s { color: #ff0000 0;}");
        testCss(".s { color: #ff0000  }");
        testCss(".s { color: #ff0000  0}");
        testCss(".s { color: #ff0000\t }");
        testCss(".s { color: #ff0000\t 0}");
        testCss(".s { color: #ff0000 \n }");

        // Whitespace at the beginning
        testCss(".s { color: \t#ff0000;}");
        testCss(".s { color: \t #ff0000;}");
        testCss(".s { color:\t #ff0000;}");
        testCss(".s { color:\n #ff0000;}");
        testCss(".s { color: \n #ff0000;}");
        testCss(".s { color: \n#ff0000;}");

        // Different lengths
        testCss(".s { color: #6;}");
        testCss(".s { color: #66;}");
        testCss(".s { color: #666;}");
        testCss(".s { color: #6666;}");
        testCss(".s { color: #66666;}");
        testCss(".s { color: #666666;}");
        testCss(".s { color: #6666666;}");
        testCss(".s { color: #66666666;}");
        testCss(".s { color: #666666666;}");

        // No prefix
        testCss(".s { color: 666666;}");
    }

    private function testCss(css: String) {
        trace("CSS: " + css.split("\n").join("\\n"));
        var style = new StyleSheet();
        style.parseCSS(css);
        trace("  string color: " + style.getStyle(".s").color);

        var t = new TextField();
        t.styleSheet = style;
        t.htmlText = "<p class=\"s\">x</p>";

        var tf = t.getTextFormat(0);
        trace("  parsed color: " + tf.color.toString(16));
    }
}
}
