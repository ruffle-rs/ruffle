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
        // display attribute on class
        var displayInline:Object = new Object();
        displayInline.display = "inline";
        var displayBlock:Object = new Object();
        displayBlock.display = "block";
        var displayNone:Object = new Object();
        displayNone.display = "none";
        style.setStyle(".di", displayInline);
        style.setStyle(".db", displayBlock);
        style.setStyle(".dn", displayNone);
        runHtmlTest('x<p class="di">a</p>y<span class="di">b</span>z<a class="di">c</a>s');
        runHtmlTest('x<p class="db">a</p>y<span class="db">b</span>z<a class="db">c</a>s');
        runHtmlTest('x<p class="dn">a</p>y<span class="dn">b</span>z<a class="dn">c</a>s');
        resetStyle();

        // display attribute on a custom tag
        var displayInline:Object = new Object();
        displayInline.display = "inline";
        var displayBlock:Object = new Object();
        displayBlock.display = "block";
        var displayNone:Object = new Object();
        displayNone.display = "none";
        style.setStyle("di", displayInline);
        style.setStyle("db", displayBlock);
        style.setStyle("dn", displayNone);
        runHtmlTest('<di>a</di>');
        runHtmlTest('<db>a</db>');
        runHtmlTest('<dn>a</dn>');
        runHtmlTest('x<di></di>y');
        runHtmlTest('x<db></db>y');
        runHtmlTest('x<dn></dn>y');
        runHtmlTest('x<di>y');
        runHtmlTest('x<db>y');
        runHtmlTest('x<dn>y');
        runHtmlTest('x<di></p>y');
        runHtmlTest('x<db></p>y');
        runHtmlTest('x<dn></p>y');
        runHtmlTest('x<di>a</di>y<db>b</db>z<dn>c</dn>s');
        resetStyle();

        // display attribute on known tags (inline)
        var displayInline:Object = new Object();
        displayInline.display = "inline";
        style.setStyle("p", displayInline);
        style.setStyle("li", displayInline);
        style.setStyle("a", displayInline);
        runHtmlTest('<p>a</p>');
        runHtmlTest('<li>a</li>');
        runHtmlTest('<a>a</a>');
        runHtmlTest('x<p>a</p>y<li>b</li>z<a>c</a>s');
        resetStyle();

        // display attribute on known tags (block)
        var displayBlock:Object = new Object();
        displayBlock.display = "block";
        style.setStyle("p", displayBlock);
        style.setStyle("li", displayBlock);
        style.setStyle("a", displayBlock);
        runHtmlTest('<p>a</p>');
        runHtmlTest('<li>a</li>');
        runHtmlTest('<a>a</a>');
        runHtmlTest('x<p>a</p>y<li>b</li>z<a>c</a>s');
        resetStyle();

        // display attribute on known tags (none)
        var displayNone:Object = new Object();
        displayNone.display = "none";
        style.setStyle("p", displayNone);
        style.setStyle("li", displayNone);
        style.setStyle("a", displayNone);
        runHtmlTest('<p>a</p>');
        runHtmlTest('<li>a</li>');
        runHtmlTest('<a>a</a>');
        runHtmlTest('x<p>a</p>y<li>b</li>z<a>c</a>s');
        resetStyle();

        // display on tag without style
        runHtmlTest('<as>a</as>');
        runHtmlTest('x<as>a</as>y<as>b</as>z');
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
