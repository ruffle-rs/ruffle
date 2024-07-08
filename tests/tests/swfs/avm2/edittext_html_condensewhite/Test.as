package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    public function Test() {
        runTests(new TextField());
    }

    function runTests(text: TextField) {
        trace("condenseWhite: " + text.condenseWhite);
        text.condenseWhite = true;
        runTest(text, '\n');
        runTest(text, '\n\n');
        runTest(text, ' ');
        runTest(text, '  ');
        runTest(text, ' \n');
        runTest(text, '\n ');
        runTest(text, '\n \tasdf \t  \n');
        runTest(text, ' test ');
        runTest(text, ' test test ');
        runTest(text, '\ntest  \n');
        runTest(text, 'test \n');
        runTest(text, 'test \ntest\n\ntest\n\n\ntest');
        runTest(text, 'test\n \ntest \n');
        runTest(text, '<b>test</b> \n');
        runTest(text, '\n <p>t\ne s\tt </p> \n');
        runTest(text, '<li>test</li>\n');
        runTest(text, '<b> \n </b>');
        runTest(text, '<b></b>\n');
        runTest(text, '<b> </b>');
        runTest(text, ' <b> </b> ');
        runTest(text, ' <p> </p> ');
        runTest(text, '<b> test </b>');
        runTest(text, '<b>\ntest\n</b>');
        runTest(text, '\n<p>test</p>\n');
        runTest(text, ' <p>test</p>  <p>test</p> ');
        runTest(text, '<p></p>\n');
        runTest(text, '<p>\n</p>');
        runTest(text, '<p>  \n  </p>\n  ');
        runTest(text, '<p> </p> ');
        runTest(text, '<p> test </p>');
        runTest(text, '<p>\ntest\n</p>');
        runTest(text, '<li></li>\n');
        runTest(text, '<li>\n</li>');
        runTest(text, '<li>test\n</li>');
        runTest(text, '<li> </li>');
        runTest(text, '<li> </li> ');
        runTest(text, '<li> test </li>');
        runTest(text, '<li>\ntest\n</li>');
        runTest(text, 'a b \xa0c \x01 \x02 \x03 d');
        runTest(text, 'a \x04 \x05 \x06 \x07 b');
        runTest(text, 'a \x08 \x09 \x0a \x0b b');
        runTest(text, 'a \x0c \x0d \x0e \x0f b');
        runTest(text, 'a \x10 \x11 \x12 \x13 b');
        runTest(text, 'a \x14 \x15 \x16 \x17 b');
        runTest(text, 'a \x18 \x19 b');
        runTest(text, '  <p>  test  </p>  ');
        runTest(text, '  <p>\n test  </p>\n ');
        runTest(text, '  <p>test</p>  ');
        runTest(text, '<p>  test  </p>');
        runTest(text, ' <b> test1 </b> <b> test2 </b> ');
        runTest(text, ' <b> test1 </b> <i> test2 </i> ');
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
        return string.replace(/\n/g, "\\n").replace(/\r/g, "\\r").replace(/Times New Roman/g, "Times");
    }
}
}
