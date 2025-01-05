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
        runInitialTests();
        runHtmlTests();
    }

    function resetStyle() {
        style = new StyleSheet();

        var classBold:Object = new Object();
        classBold.fontWeight = "bold";
        style.setStyle(".classBold", classBold);

        var classRed:Object = new Object();
        classRed.color = "#FF0000";
        style.setStyle(".classRed", classRed);

        var classGreen:Object = new Object();
        classGreen.color = "#00FF00";
        style.setStyle(".classGreen", classGreen);

        var classRedBold:Object = new Object();
        classRedBold.color = "#FF0000";
        classRedBold.fontWeight = "bold";
        style.setStyle(".classRedBold", classRedBold);
    }

    function runInitialTests() {
        var text: TextField = null;

        trace("Getter & setter:");
        text = newTextField();
        var s = new StyleSheet();
        text.styleSheet = s;

        trace("  Style before (1): " + s.styleNames);
        trace("  Style before (2): " + text.styleSheet.styleNames);

        var classRed:Object = new Object();
        classRed.color = "#FF0000";
        s.setStyle(".classRed", classRed);

        trace("  Style after (1):  " + s.styleNames);
        trace("  Style after (2):  " + text.styleSheet.styleNames);

        trace("Setting style after HTML:");
        text = newTextField();
        text.htmlText = '<span class="classRed">a</span>';
        trace("  HTML before:      " + escape(text.htmlText));
        text.styleSheet = style;
        trace("  HTML after style: " + escape(text.htmlText));
        text.htmlText = '<span class="classRed">a</span>';
        trace("  HTML after set:   " + escape(text.htmlText));
        text.styleSheet = null;
        trace("  HTML no style:    " + escape(text.htmlText));

        trace("Setting text after stylized HTML:");
        text = newTextField();
        text.styleSheet = style;
        text.htmlText = '<span class="classRed">a</span>';
        trace("  HTML after style: " + escape(text.htmlText));
        trace("  Text after style: " + escape(text.text));
        text.text = 'b';
        trace("  HTML after set:   " + escape(text.htmlText));
        trace("  Text after set:   " + escape(text.text));
        text.styleSheet = null;
        trace("  HTML after reset: " + escape(text.htmlText));
        trace("  Text after reset: " + escape(text.text));
        text.text = 'c';
        trace("  HTML after set:   " + escape(text.htmlText));
        trace("  Text after set:   " + escape(text.text));

        trace("Setting HTML in text:");
        text = newTextField();
        text.styleSheet = style;
        text.text = '<span class="classRed">a</span>';
        trace("  HTML after style: " + escape(text.htmlText));
        trace("  Text after style: " + escape(text.text));
        text.styleSheet = null;
        trace("  HTML after reset: " + escape(text.htmlText));
        trace("  Text after reset: " + escape(text.text));

        trace("Modifying CSS after parsing HTML:");
        text = newTextField();
        var s = new StyleSheet();
        text.styleSheet = s;
        text.htmlText = '<span class="classRed">a</span>';
        trace("  Style (original): " + text.styleSheet.styleNames);
        printTextRuns(text.getTextRuns());
        var classRed:Object = new Object();
        classRed.color = "#FF0000";
        s.setStyle(".classRed", classRed);
        trace("  Style (after modifying CSS): " + text.styleSheet.styleNames);
        printTextRuns(text.getTextRuns());
        text.htmlText = '<span class="classRed">a</span>';
        trace("  Style (after updating HTML to the same value): " + text.styleSheet.styleNames);
        printTextRuns(text.getTextRuns());
        text.htmlText = '<span class="classRed">b</span>';
        trace("  Style (after updating HTML): " + text.styleSheet.styleNames);
        printTextRuns(text.getTextRuns());
        text.styleSheet = s;
        trace("  Style (after updating CSS): " + text.styleSheet.styleNames);
        printTextRuns(text.getTextRuns());
        text.htmlText = '<span class="classRed">c</span>';
        trace("  Style (after updating HTML): " + text.styleSheet.styleNames);
        printTextRuns(text.getTextRuns());
        var classRed:Object = new Object();
        classRed.color = "#00FF00";
        s.setStyle(".classRed", classRed);
        text.styleSheet = s;
        trace("  Style (after updating CSS without HTML): " + text.styleSheet.styleNames);
        printTextRuns(text.getTextRuns());

        trace("Modifying text after removing CSS:");
        text = newTextField();
        var s = new StyleSheet();
        text.text = '<b>1</b>';
        text.styleSheet = s;
        text.styleSheet = null;
        text.text = '<b>2</b>';
        trace("  Text after: " + text.text);
        trace("  HTML after: " + text.htmlText);

        trace("Modifying text after removing CSS with HTML:");
        text = newTextField();
        var s = new StyleSheet();
        text.htmlText = '<b>1</b>';
        text.styleSheet = s;
        text.styleSheet = null;
        text.text = '<b>2</b>';
        trace("  Text after: " + text.text);
        trace("  HTML after: " + text.htmlText);

        trace("Updating CSS and resetting it:");
        text = newTextField();
        text.styleSheet = new StyleSheet();
        text.htmlText = '<span class="classred">x</span>';
        printTextRuns(text.getTextRuns());
        text.styleSheet = style;
        text.styleSheet = null;
        printTextRuns(text.getTextRuns());
    }

    function runHtmlTests() {
        // Unknown classes
        runHtmlTest('<span class="unknownClass">a</span>');
        runHtmlTest('<span class="unknownClass">a</span><span class="unknownClass">b</span>');

        // Basic spans
        runHtmlTest('<span class="classBold">a</span>');
        runHtmlTest('<span class="classRed">a</span>');
        runHtmlTest('<span class="classGreen">a</span>');
        runHtmlTest('<span class="classRedBold">a</span>');

        // Nesting & overlapping classes
        runHtmlTest('<span class="classRed">a<span class="classGreen">b</span></span>');
        runHtmlTest('<span class="classRed">a<span class="classRed">b</span></span>');
        runHtmlTest('<span class="classRed">a<span class="classBold">b</span></span><span class="classRedBold">c</span>');

        // Multiple classes?
        runHtmlTest('<span class="classRed classBold">a</span><span class="classRedBold">b</span>');

        // Spaces
        runHtmlTest('<span class="  classRed">a</span>');
        runHtmlTest('<span class="classRed  ">a</span>');
        runHtmlTest('<span class="   classRed  ">a</span>');

        // Dashes in names
        var classMagenta:Object = new Object();
        classMagenta.color = "#FF00FF";
        style.setStyle(".class-magenta", classMagenta);
        runHtmlTest('<span class="class-magenta">a</span><span class="classmagenta">b</span><span class="class magenta">c</span>');

        // Case sensitivity
        runHtmlTest('<span class="classred">a</span>');
        runHtmlTest('<span class="CLASSRED">a</span>');
        runHtmlTest('<span class="ClassRed">a</span>');

        // Importance of styles
        runHtmlTest('<font color="#00FF00">a<span class="classRed">b</span></font>');
        runHtmlTest('<span class="classRed">a<font color="#00FF00">b</span></font>');

        // Class on paragraph
        runHtmlTest('<p class="classRed">a</p><p class="classBold">b</p>');
        runHtmlTest('<p>a</p><p class="classBold">b</p>');
        runHtmlTest('<p class="classRed">a</p><p>b</p>');
        runHtmlTest('<p>a</p><p>b</p>');

        // Styling elements
        var classBlue:Object = new Object();
        classBlue.color = "#0000FF";
        style.setStyle("a", classBlue);
        style.setStyle("b", classBlue);
        style.setStyle("u", classBlue);
        style.setStyle("textformat", classBlue);
        style.setStyle("font", classBlue);
        style.setStyle("p", classBlue);
        style.setStyle("style", classBlue);
        style.setStyle("i", classBlue);
        style.setStyle("li", classBlue);
        runHtmlTest(' <a>a</a> ');
        runHtmlTest(' <b>b</b> ');
        runHtmlTest(' <u>u</u> ');
        runHtmlTest(' <textformat>textformat</textformat> ');
        runHtmlTest(' <font>font</font> ');
        runHtmlTest(' <p>p</p> ');
        runHtmlTest(' <span>span</span> ');
        runHtmlTest(' <i>i</i> ');
        runHtmlTest(' <li>li</li> ');
        runHtmlTest(' <a class="classRed">a</a> ');
        resetStyle();

        // Style vs attribute preference (font)
        var classBlue:Object = new Object();
        classBlue.color = "#0000FF";
        style.setStyle("font", classBlue);
        runHtmlTest(' <font>a</font> ');
        runHtmlTest(' <font color="#00FF00">a</font> ');
        resetStyle();

        // Style vs attribute preference (p)
        var classPa:Object = new Object();
        classPa.textAlign = "right";
        var classPb:Object = new Object();
        classPb.textAlign = "center";
        style.setStyle("classp", classPa);
        style.setStyle("p", classPb);
        runHtmlTest(' <p>a</p> ');
        runHtmlTest(' <p class="classp">a</p> ');
        runHtmlTest(' <p align="justify">a</p> ');
        runHtmlTest(' <p class="classp" align="justify">a</p> ');
        runHtmlTest(' <p align="justify" class="classp">a</p> ');
        resetStyle();

        // Style vs attribute preference (span)
        var classSpan:Object = new Object();
        classSpan.color = "#0000FF";
        style.setStyle("span", classSpan);
        runHtmlTest(' <span>a</span> ');
        runHtmlTest(' <span class="colorRed">a</span> ');
        resetStyle();

        // Star?
        var classAll:Object = new Object();
        classAll.color = "#0000FF";
        style.setStyle("*", classAll);
        runHtmlTest(' <span>a</span> ');
        runHtmlTest(' hello <b>b</b> ');
        resetStyle();

        // Specific tags with styles
        var classAll:Object = new Object();
        classAll.color = "#0000FF";
        style.setStyle("p.classspec", classAll);
        runHtmlTest(' <p class="classspec">a</p> <span class="classspec">b</span> ');
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
