package {
import flash.display.*;
import flash.text.*;

public class Test extends Sprite {
    private var style:StyleSheet;

    public function Test() {
        runTests();
    }

    function runTests() {
        resetStyle();
        runHtmlTests();
    }

    function resetStyle() {
        style = new StyleSheet();
    }

    function runHtmlTests() {
        var colorRed:Object = new Object();
        colorRed.color = 0xFF0000;
        var colorBlue:Object = new Object();
        colorBlue.color = 0xFF;
        style.setStyle("red", colorRed);
        style.setStyle("blue", colorBlue);
        runHtmlTest('<red>r</red>');
        runHtmlTest('<blue>b</blue>');
        runHtmlTest('<blue>b<red>r</red></blue>');
        runHtmlTest('<red>r<blue>b</blue></red>');
        resetStyle();

        var classRedBold:Object = new Object();
        classRedBold.color = "#FF0000";
        classRedBold.fontWeight = "bold";
        var colorBlue:Object = new Object();
        colorBlue.color = 0xFF;
        style.setStyle("red", classRedBold);
        style.setStyle("blue", colorBlue);
        runHtmlTest('<red>r</red>');
        runHtmlTest('<blue>b</blue>');
        runHtmlTest('<blue>b<red>r</red></blue>');
        runHtmlTest('<red>r<blue>b</blue></red>');
        resetStyle();
    }

    function runHtmlTest(html: String) {
        var text = newTextField();
        text.styleSheet = style;

        trace("======================================");
        trace("  Setting HTML: " + escape(html));
        text.htmlText = html;
        trace("  HTML get:     " + escape(text.htmlText));
        trace("  Text get:     " + escape(text.text));
        printTextRuns(text.getTextRuns());
        text.styleSheet = null;
        trace("  Style reset:  " + escape(text.htmlText));
        printTextRuns(text.getTextRuns());
    }

    private function printTextRuns(runs: Array) {
        trace("  Text runs (" + runs.length + "):");
        for each (var run in runs) {
            trace("    from " + run.beginIndex + " to " + run.endIndex + ": " + describeTextFormat(run.textFormat));
        }
    }

    private function describeTextFormat(tf: TextFormat): String {
        return "size=" + tf.size +
                ", blockIndent=" + tf.blockIndent +
                ", font=" + tf.font +
                ", align=" + tf.align +
                ", leading=" + tf.leading +
                ", display=" + tf.display +
                ", kerning=" + tf.kerning +
                ", leftMargin=" + tf.leftMargin +
                ", rightMargin=" + tf.rightMargin +
                ", color=" + tf.color +
                ", bold=" + tf.bold +
                ", italic=" + tf.italic +
                ", bullet=" + tf.bullet +
                ", underline=" + tf.underline;
    }

    private function escape(string: String): String {
        return string.replace(/\n/g, "\\n").replace(/\r/g, "\\r");
    }

    private function newTextField(): TextField {
        var text = new TextField();
        text.defaultTextFormat = new TextFormat("FontName", 10);
        return text;
    }
}
}
