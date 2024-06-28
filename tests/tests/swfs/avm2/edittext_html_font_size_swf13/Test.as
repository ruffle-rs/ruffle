package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    public function Test() {
        runTests(new TextField());
    }

    function runTests(text: TextField) {
        trace('==== font sizes');
        runTest(text, '<font size="+1">text</font>');
        runTest(text, '<font size="-1">text</font>');
        runTest(text, '<font size="12">text</font><font size="+1">text</font>');
        runTest(text, '<font size="12">text</font><font size="-1">text</font>');
        runTest(text, '<font size="12">text<font size="+1">text</font></font>');
        runTest(text, '<font size="12">text<font size="-1">text</font></font>');
        runTest(text, '<font size="12">text<font size="+1">text<font size="+1">text<font size="+1">text</font></font></font></font>');
        runTest(text, '<font size="12">text<font size="-1">text<font size="+1">text<font size="-1">text</font></font></font></font>');
        runTest(text, '<font size="1.2">text</font>');
        runTest(text, '<font size="1.2">text<font size="+0.2">text</font></font>');
        runTest(text, '<font size="1.2">text<font size="-0.2">text</font></font>');
        runTest(text, '<font size="1.9">text<font size="+0.2">text</font></font>');
        runTest(text, '<font size="1.other text">text</font>');
        runTest(text, '<font size="not a number">text</font>');
        runTest(text, '<font size="13">text<font size="not a number">text</font></font>');
        runTest(text, '<font size="13">text<font size="+not a number">text</font></font>');
        runTest(text, '<font size="13">text<font size="-not a number">text</font></font>');
        runTest(text, '<font size="+not a number">text</font>');
        runTest(text, '<font size="-not a number">text</font>');
        // how high can we go?
        runTest(text, '<font size="32">text</font><font size="64">text</font><font size="128">text</font><font size="256">text</font><font size="65536">text</font>');
        runTest(text, '<font size="126">text<font size="+1">text<font size="+1">text</font></font></font>');
        // how low can we go?
        runTest(text, '<font size="1">text</font><font size="0">text<font size="-1">text</font></font>');
        runTest(text, '<font size="1+1">text</font><font size="1-1">text<font size="-1+1">text</font></font>');
        runTest(text, '<font size="+">text</font><font size="-">text</font>');
    }

    function runTest(text: TextField, html: String) {
        trace("    HTML set:    " + escape(html));

        text.multiline = false;
        text.htmlText = html;
        var lastHtml = text.htmlText;
        trace("    HTML get:    " + escape(lastHtml));
        trace("    Text get:    " + escape(text.text));
        printTextRuns(text.getTextRuns());

        text.multiline = true;
        text.htmlText = html;
        if (lastHtml === text.htmlText) {
            trace("    HTML get ml: <!-- the same -->");
        } else {
            trace("    HTML get ml: " + escape(text.htmlText));
        }
        trace("    Text get:    " + escape(text.text));
        printTextRuns(text.getTextRuns());
        trace("    ===============");
    }

    private function printTextRuns(runs: Array) {
        trace("    Text runs (" + runs.length + "):");
        for each (var run in runs) {
            trace("      from " + run.beginIndex + " to " + run.endIndex + ": " + describeTextFormat(run.textFormat));
        }
    }

    private function describeTextFormat(tf: TextFormat): String {
        return "size=" + tf.size +
                ", blockIndent=" + tf.blockIndent +
                ", font=" + tf.font.replace(/Times New Roman/g, "Times") +
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
        return string.replace(/\n/g, "\\n").replace(/\r/g, "\\n").replace(/Times New Roman/g, "Times");
    }
}
}
