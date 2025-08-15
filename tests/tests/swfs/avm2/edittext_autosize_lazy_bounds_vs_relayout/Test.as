package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.text.TextLineMetrics;

/**
 * This test verifies the relation between relayout and autoSize bounds update.
 *
 * First, we define a metric to observe (M)—i.e. a property or a behavior of a text field
 * that we want to verify how it changes in time, which is somehow related to relayout.
 * I.e. textWidth or textHeight.
 *
 * Then, we know that some operations don't cause bounds update, based on other tests.
 * These operations are (in our case) setting autoSize and setting text.
 * The main idea is to modify autoSize, set text, modify autoSize again,
 * print x,y,w,h, and at the same time check whether M changes.
 *
 * This way, we can verify whether a relayout took place (when M changes),
 * and whether autoSize updated the bounds (when x,y,w,h change).
 *
 * The results show that relayout may take place without autoSize updating the bounds.
 * It means that relayout *is not tied* to autoSize updating the bounds.
 */
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    public function Test() {
        stage.scaleMode = "noScale";

        performTests();

        trace("Tests finished");
    }

    private function performTests():void {
        performTest("maxScrollH", function(text:TextField):* {
            return text.maxScrollH;
        });
        performTest("maxScrollV", function(text:TextField):* {
            return text.maxScrollV;
        });
        performTest("bottomScrollV", function(text:TextField):* {
            return text.bottomScrollV;
        });
        performTest("textWidth", function(text:TextField):* {
            return text.textWidth;
        });
        performTest("textHeight", function(text:TextField):* {
            return text.textHeight;
        });
        performTest("length", function(text:TextField):* {
            return text.length;
        });
        performTest("numLines", function(text:TextField):* {
            return text.numLines;
        });
        performTest("getLineMetrics", function(text:TextField):* {
            return "" + text.getLineMetrics(0).width + ";" +
                (text.numLines > 1 ? text.getLineMetrics(1).width : "?");
        });
        performTest("getLineOffset", function(text:TextField):* {
            return "" + text.getLineOffset(0) + ";" +
                (text.numLines > 1 ? text.getLineOffset(1) : "?");
        });
        performTest("getLineText", function(text:TextField):* {
            function lineToString(line:String): String {
                return line.replace("\r", "\\n").replace("\n", "\\n");
            }
            return "" + lineToString(text.getLineText(0)) + ";" +
                (text.numLines > 1 ? lineToString(text.getLineText(1)) : "?");
        });
        performTest("getLineLength", function(text:TextField):* {
            return "" + text.getLineLength(0) + ";" +
                (text.numLines > 1 ? text.getLineLength(1) : "?");
        });
        performTest("getLineIndexAtPoint", function(text:TextField):* {
            return "" + text.getLineIndexAtPoint(2, 2) + ";" + text.getLineIndexAtPoint(30, 30);
        });
        performTest("getLineIndexOfChar", function(text:TextField):* {
            return "" + text.getLineIndexOfChar(0) + ";" + text.getLineIndexOfChar(15);
        });
        performTest("getCharBoundaries", function(text:TextField):* {
            return "" + text.getCharBoundaries(0) + ";" + text.getCharBoundaries(5);
        });
        performTest("getCharIndexAtPoint", function(text:TextField):* {
            return "" + text.getCharIndexAtPoint(2, 2) + ";" + text.getCharIndexAtPoint(75, 4);
        });
    }

    private function performTest(desc: String, propReader: Function):void {
        var text;
        text = new TextField();
        text.width = 100;
        text.height = 60;
        var tf = new TextFormat();
        tf.size = 20;
        tf.font = "TestFont";
        text.defaultTextFormat = tf;
        text.embedFonts = true;

        trace("Test: " + desc);
        trace("  value: " + propReader(text));
        text.autoSize = "right";
        trace("  value: " + propReader(text));
        text.text = "aaaaaaaaaa\na\na\na\na\na\na\na\na\na\n";
        trace("  value: " + propReader(text));
        text.autoSize = "left";
        trace("  value: " + propReader(text));
        trace("  x,y,w,h: " + text.x + "," + text.y + "," + text.width + "," + text.height);
        trace("  value: " + propReader(text));
    }
}
}
