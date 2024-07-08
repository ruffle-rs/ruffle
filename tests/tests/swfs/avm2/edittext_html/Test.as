package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    public function Test() {
        runTests(new TextField());
    }

    function runTests(text: TextField) {
        trace("Default text format: " + describeTextFormat(text.defaultTextFormat));
        trace("Default multiline: " + text.multiline);

        trace('==== whitespace only');
        runTest(text, '\n');
        runTest(text, '\n\n');
        runTest(text, ' ');
        runTest(text, '  ');
        runTest(text, ' \n');
        runTest(text, '\n ');

        trace('==== behaviors of newlines and whitespace');
        runTest(text, ' test ');
        runTest(text, ' test test ');
        runTest(text, '\ntest\n');
        runTest(text, 'test\n');
        runTest(text, 'test\ntest\n\ntest\n\n\ntest');
        runTest(text, 'test\n\ntest\n');
        runTest(text, '<b>test</b>\n');
        runTest(text, '<p>test</p>\n');
        runTest(text, '<li>test</li>\n');
        runTest(text, '<b>\n</b>');
        runTest(text, '<b></b>\n');
        runTest(text, '<b> </b>');
        runTest(text, ' <b> </b> ');
        runTest(text, '<b> test </b>');
        runTest(text, '<b>\ntest\n</b>');
        runTest(text, '\n<p>test</p>\n');
        runTest(text, ' <p>test</p>  <p>test</p> ');
        runTest(text, '<a href="http://example.com"></a>\n');
        runTest(text, '<p></p>\n');
        runTest(text, '<p>\n</p>');
        runTest(text, '<p>\n</p>\n');
        runTest(text, '<p> </p>');
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
        runTest(text, '<a href="http://example.com">test</a>\n');
        runTest(text, '<textformat>test</textformat>\n');
        runTest(text, '<textformat leading="1">test</textformat>\n');
        runTest(text, '<textformat leading="1"><p>test</p></textformat>\n');
        runTest(text, '<textformat leading="1"><p><font color="#111111">test</font></p></textformat>\n');

        trace('==== empty tags');
        runTest(text, '<i></i>');
        runTest(text, '<u></u>');
        runTest(text, '<b></b>');
        runTest(text, '<p></p>');
        runTest(text, '<li></li>');
        runTest(text, '<font></font>');
        runTest(text, '<font color="#111111"></font>');
        runTest(text, '<textformat></textformat>');
        runTest(text, '<textformat leading="1"></textformat>');

        trace('==== tag order');
        runTest(text, '<b><i><u>test</u></i></b>');
        runTest(text, '<u><i><b>test</b></i></u>');
        runTest(text, '<i><u><b>test</b></u></i>');
        runTest(text, '<b><i><u>test</u> <u>test</u></i> <i><u>test</u> <u>test</u></i></b>');
        runTest(text, '<i><u><b>test</b> <b>test</b></u> <u><b>test</b> <b>test</b></u></i>');
        runTest(text, '<b><font color="#010101">test</font></b>');
        runTest(text, '<font color="#010101"><textformat leading="0">test</textformat></font>');
        runTest(text, '<i><font color="#010101"><textformat leading="0">test</textformat></font></i>');
        runTest(text, '<i><a href="http://example.com/"><font color="#010101"><textformat leading="0"><b>test</b></textformat></font></a></i>');

        trace("==== merging tags: same tags");
        runTest(text, ' <p>test</p> <p>test</p> ');
        runTest(text, '<p>test</p><p>test</p>');
        runTest(text, ' <font color="#010101">test</font> <font color="#010101">test</font> ');
        runTest(text, '<font color="#010101">test</font><font color="#010101">test</font>');
        runTest(text, ' <li>test</li> <li>test</li> ');
        runTest(text, '<li>test</li><li>test</li>');
        runTest(text, ' <b>test</b> <b>test</b> ');
        runTest(text, '<b>test</b><b>test</b>');
        runTest(text, ' <i>test</i> <i>test</i> ');
        runTest(text, '<i>test</i><i>test</i>');
        runTest(text, ' <u>test</u> <u>test</u> ');
        runTest(text, '<u>test</u><u>test</u>');
        runTest(text, ' <a href="http://example.com/">test</a> <a href="http://example.com/">test</a> ');
        runTest(text, '<a href="http://example.com/">test</a><a href="http://example.com/">test</a>');
        runTest(text, ' <textformat leading="0">test</textformat> <textformat leading="0">test</textformat> ');
        runTest(text, '<textformat leading="0">test</textformat><textformat leading="0">test</textformat>');

        trace('==== merging tags: same tags, different attributes');
        runTest(text, ' <p align="right">test</p> <p align="left">test</p> ');
        runTest(text, '<p align="right">test</p><p align="left">test</p>');
        runTest(text, ' <font color="#010101\">test</font> <font color="#020202">test</font> ');
        runTest(text, '<font color="#010101\">test</font><font color="#020202">test</font>');
        runTest(text, ' <a href="http://example.com/1">test</a> <a href="http://example.com/2">test</a> ');
        runTest(text, '<a href="http://example.com/1">test</a><a href="http://example.com/2">test</a>');
        runTest(text, ' <textformat leading="0">test</textformat> <textformat leading="1">test</textformat> ');
        runTest(text, '<textformat leading="0">test</textformat><textformat leading="1">test</textformat>');
        runTest(text, '<textformat leftmargin="1">test</textformat><textformat rightmargin="1">test</textformat>');
        runTest(text, '<textformat leftmargin="1"><p>test</p></textformat><textformat rightmargin="1"><p>test</p></textformat>');

        trace('==== font stack');
        runTest(text, '<font face="Noto Sans"><font color="#aaaaaa">test</font> <font color="#bbbbbb">test</font></font>');
        runTest(text, '<font face="Noto Sans"><font color="#aaaaaa">test</font> <font color="#aaaaaa">test</font> <font color="#bbbbbb">test</font></font>');
        runTest(text, '<font color="#aaaaaa"><font face="Noto Sans">test</font> <font face="Noto Sans">test</font></font>');
        runTest(text, '<font color="#aaaaaa"><font face="Noto Sans">test</font> <font face="Noto Sans">test</font></font> <font color="#bbbbbb"><font face="Noto Sans">test</font> <font face="Noto Sans">test</font></font>');
        runTest(text, '<font color="#aaaaaa" face="Noto Sans">test</font> <font color="#aaaaaa" face="Noto Sans">test</font>');
        runTest(text, '<font color="#aaaaaa" face="Noto Sans">test<font color="#000000"> </font>test</font>');
        runTest(text, '<font face="Noto Sans"><font color="#aaaaaa">test</font></font><font face="Noto Sans">test</font>');
        runTest(text, '<font face="Noto Sans" color="#aaaaaa">test</font><font face="Noto Sans" color="#bbbbbb">test</font>');
        runTest(text, '<font face="Noto Sans">test <font color="#aaaaaa">test</font> test</font>');
        runTest(text, '<font face="Noto Sans">test <font color="#aaaaaa">test</font><font color="#bbbbbb">test</font></font>');
        runTest(text, '<font face="Noto Sans">test <font color="#aaaaaa">test</font><font size="16">test</font></font>');
        runTest(text, '<font face="Noto Sans">test <font color="#aaaaaa">test</font><font size="16">t<b>e</b>st</font></font>');
        runTest(text, '<font face="Noto Sans">te<b>st <font color="#aaaaaa">test</font><font size="16">test</font></b></font>');
        runTest(text, '<font face="Noto Sans">test <font color="#aaaaaa">test</font><font size="16">t<p>e</p>st</font></font>');
        runTest(text, '<font face="Noto Sans">test <font color="#aaaaaa">test <font size="16">test</font></font> test</font>');
        runTest(text, '<a href="http://example.com/"><font color="#aaaaaa">test <font size="16">test</font></font> test</a>');

        trace('==== font multiline');
        runTest(text, '<font color="#aaaaaa" face="Noto Sans">test</font>\n<font color="#aaaaaa" face="Noto Sans">test</font>');
        runTest(text, '<font color="#aaaaaa" face="Noto Sans">test\ntest</font>');

        trace('==== br behavior');
        runTest(text, 'line 1 <br> line 2');
        runTest(text, '<p>line 1 <br> line 2</p>');
        runTest(text, '<li>line 1 <br> line 2</li>');
        runTest(text, '<p>line 1</p> <br> <p>line 2</p>');
        runTest(text, '<p>line 1</p><br><p>line 2</p>');
        runTest(text, '<b>line 1 <br> line 2</b>');
        runTest(text, '<a href="http://example.com">line 1 <br> line 2</a>');
        runTest(text, '<font color="#aaaaaa" face="Noto Sans">test<br>test</font>');
        runTest(text, '<font kerning="1"><i><br></i>text</font>');
        runTest(text, 'test <b> test <br> test </b> test <br> test');
        runTest(text, '<font color="#010101">texttext<b><br></b></font>');
        runTest(text, 'text<li><br><textformat leftmargin="2"></textformat></li>');
        runTest(text, 'text<br><textformat indent="5"><textformat rightmargin="3"><br></textformat></textformat>');

        trace('==== textformat behavior');
        runTest(text, '<textformat></textformat>\n');
        runTest(text, '<textformat leading="1"></textformat>\n');
        runTest(text, '<textformat leading="1"><p></p></textformat>\n');
        runTest(text, '<textformat leading="1"><p><font color="#111111"></font></p></textformat>\n');
        runTest(text, '<textformat leading="1"><p><font color="#111111"><b></b></font></p></textformat>\n');
        runTest(text, '<TEXTFORMAT LEADING="1"><P ALIGN="LEFT"><FONT FACE="Some Font" SIZE="10" COLOR="#000000" LETTERSPACING="0" KERNING="0"></FONT></P></TEXTFORMAT>\n');
        runTest(text, 'text<textformat rightmargin="3">text</textformat><textformat rightmargin="2">text</textformat><br><textformat rightmargin="4">text</textformat><textformat rightmargin="5">text</textformat>');
        runTest(text, 'text<textformat leftmargin="2"><li><a href="http://example.com" target="_blank"><br></a></li></textformat>');
        runTest(text, '<textformat leading="1">a<textformat rightmargin="2">b</textformat>c</textformat>d');
        runTest(text, 'test<textformat leading="1">a<textformat rightmargin="2">b</textformat>c</textformat>d');

        trace('==== p and li behavior');
        runTest(text, '<i>text<li>text</li></i>');
        runTest(text, '<i>text<li></li></i>');
        runTest(text, '<font color="#010101">text<li></li></font>');
        runTest(text, '<p align="right">text<li></li></p>');
        runTest(text, 'text<li><li></li></li>');
        runTest(text, 'text<li><li><li></li></li></li>');
        runTest(text, 'text<li>a<li>b<li>c</li>d</li>e</li>f');
        runTest(text, 'text<li><li>test</li></li>');
        runTest(text, 'text<li>a<li>test</li></li>');
        runTest(text, 'text<li>a<b></b><li>test</li></li>');
        runTest(text, '<li>test<b></b><li>test</li></li>');
        runTest(text, '<p>test<b></b><p>test</p></p>');
        runTest(text, '<li>test<u></u><li>test</li></li>');
        runTest(text, '<p>test<u></u><p>test</p></p>');
        runTest(text, '<li>test<i></i><li>test</li></li>');
        runTest(text, '<p>test<i></i><p>test</p></p>');
        runTest(text, '<li>test<font color="#121212"></font><li>test</li></li>');
        runTest(text, '<p>test<font color="#121212"></font><p>test</p></p>');
        runTest(text, '<p>test<font color="#121212"></font><p>test<font color="#131313"></font><p>test</p></p></p>');
        runTest(text, '<p>test<font color="#121212"></font><p>test<font color="#131313"></font><p>test<font color="#141414"></font></p></p></p>');
        runTest(text, '<p>test<font color="#121212"></font><p>test<font color="#131313"></font><p>test<font color="#141414"></font></p></p></p><p>test</p>');
        runTest(text, '<p>test<font color="#121212"></font><p>test</p><font color="#131313"><p>test</p></font>');
        runTest(text, '<p>test<b></b><p>test</p></p>');
        runTest(text, '<p>test<b></b><p>test<i></i><p>test</p></p></p>');
        runTest(text, '<p>test<b></b><p>test<i></i><p>test<u></u></p></p></p>');
        runTest(text, '<p>test<font color="#121212"></font><font color="#131313"><font color="#141414"></font></font><p>test</p></p>');
        runTest(text, '<li>test<textformat leading="1"></textformat><li>test</li></li>');
        runTest(text, 'text<li></li>');
        runTest(text, '<p></p><p></p>test<p></p><p></p>');
        runTest(text, '<p></p><p></p>test<p></p><p></p><p></p><p></p><p></p>');
        runTest(text, '<p>a<p>b</p>c<p>d</p>e</p>');
        runTest(text, '<li>test<b></b></li>');
        runTest(text, '<li>test<u></u></li>');
        runTest(text, '<li>test<i></i></li>');
        runTest(text, '<p>test<b></b></p>');
        runTest(text, '<p>test<u></u></p>');
        runTest(text, '<p>test<i></i></p>');
        runTest(text, '<i><p align="right">text</p></i>');
        runTest(text, '<b><p align="right"></p></b>');
        runTest(text, '<b><p align="right">text</p></b>');
        runTest(text, '<u><p align="right"></p></u>');
        runTest(text, '<u><p align="right">text</p></u>');
        runTest(text, '<a href="http://example.com"><p align="right"></p></a>');
        runTest(text, '<a href="http://example.com"><p align="right">text</p></a>');
        runTest(text, '<textformat leading="1"><p align="right"></p></textformat>');
        runTest(text, '<textformat leading="1"><p align="right">text</p></textformat>');
        runTest(text, '<li><font color="#010101">text</font></li>');
        runTest(text, '<p></p>\n <li>text</li>');
        runTest(text, '<li>test</li>');
        runTest(text, '<u><li>test</li></u>');
        runTest(text, '<li> </li>');
        runTest(text, 'test<li>test</li>');
        runTest(text, 'test<li><i>test</i></li>');
        runTest(text, 'test<li><i><p>test</p></i></li>');
        runTest(text, '<li>test</li>\n');
        runTest(text, '<li>test</li><p>test 2</p>');
        runTest(text, '<p><li>test</li></p>');
        runTest(text, '<li><p>test</p></li>');
        runTest(text, '<p>test</p>\n');
        runTest(text, '<p>test</p>a\n');
        runTest(text, '<p>test</p>\na');
        runTest(text, '<p>test</p>\n<p>test</p>');
        runTest(text, '<p>test</p>a\n<p>test</p>');
        runTest(text, '<p>test</p>\na<p>test</p>');
        runTest(text, '<p>test</p>\n<li>test</li>');
        runTest(text, '<p>test</p>a\n<li>test</li>');
        runTest(text, '<p>test</p>\na<li>test</li>');
        runTest(text, '<li>test</li>\n');
        runTest(text, '<li>test</li>a\n');
        runTest(text, '<li>test</li>\na');
        runTest(text, '<li>test</li>\n<li>test</li>');
        runTest(text, '<li>test</li>a\n<li>test</li>');
        runTest(text, '<li>test</li>\na<li>test</li>');
        runTest(text, '<li>test</li>\n<p>test</p>');
        runTest(text, '<li>test</li>a\n<p>test</p>');
        runTest(text, '<li>test</li>\na<p>test</p>');
        runTest(text, '<i>text<li>text</li></i>');
        runTest(text, '<font color="#010101"><textformat rightmargin="3"><p><li></li></p></textformat></font>');
        runTest(text, '<a href="http://example.com"><p></p></a>');
        runTest(text, '<a href="http://example.com" target="_blank">\n<textformat tabstops="1,2,3"><p><li></li></p></textformat></a>');
        runTest(text, '<a href="http://example.com" target="_blank"><textformat tabstops="1,2,3"><p><li></li></p></textformat></a>');
        runTest(text, '<a href="http://example.com" target="_blank">\n<textformat tabstops="1,2,3"><p></p></textformat></a>');
        runTest(text, '<a href="http://example.com" target="_blank">\n<textformat tabstops="1,2,3"><li></li></textformat></a>');

        trace('==== p and li merging');
        runTest(text, '<p>first</p><li>second</li>');
        runTest(text, ' <p>first</p> <li>second</li> ');
        runTest(text, '<li>first</li><p>second</p>');
        runTest(text, ' <li>first</li> <p>second</p> ');
        runTest(text, '<p>first</p><p>second</p>');
        runTest(text, ' <p>first</p> <p>second</p> ');
        runTest(text, '<li>first</li><li>second</li>');
        runTest(text, ' <li>first</li> <li>second</li> ');
        runTest(text, '<p></p><li>test</li>');
        runTest(text, '<li></li><p>test</p>');
        runTest(text, '<li></li><p></p><li>test</li>');
        runTest(text, '<p>a<p>b<p>c</p>d</p>e</p>');

        trace('==== p and li nesting');
        // there are some weird things going on here,
        // hence a lot of tests to catch some pattern...
        runTest(text, '<p><p>test</p></p>');
        runTest(text, '<p>a <p>test</p> b</p>');
        runTest(text, '<p><li>test</li></p>');
        runTest(text, '<p>a <li>test</li> b</p>');
        runTest(text, '<li><p>test</p></li>');
        runTest(text, '<li>a <p>test</p> b</li>');
        runTest(text, '<li><li>test</li></li>');
        runTest(text, '<li>a <li>test</li> b</li>');
        runTest(text, '<p><p><p>test</p></p></p>');
        runTest(text, '<p><p><li>test</li></p></p>');
        runTest(text, '<p><li><p>test</p></li></p>');
        runTest(text, '<p><li><li>test</li></li></p>');
        runTest(text, '<li><p><p>test</p></p></li>');
        runTest(text, '<li><p><li>test</li></p></li>');
        runTest(text, '<li><li><p>test</p></li></li>');
        runTest(text, '<li><li><li>test</li></li></li>');
        runTest(text, '<p><p><p><p>test</p></p></p></p>');
        runTest(text, '<p><p><p><li>test</li></p></p></p>');
        runTest(text, '<p><p><li><p>test</p></li></p></p>');
        runTest(text, '<p><p><li><li>test</li></li></p></p>');
        runTest(text, '<p><li><p><p>test</p></p></li></p>');
        runTest(text, '<p><li><p><li>test</li></p></li></p>');
        runTest(text, '<p><li><li><p>test</p></li></li></p>');
        runTest(text, '<p><li><li><li>test</li></li></li></p>');
        runTest(text, '<li><p><p><p>test</p></p></p></li>');
        runTest(text, '<li><p><p><li>test</li></p></p></li>');
        runTest(text, '<li><p><li><p>test</p></li></p></li>');
        runTest(text, '<li><p><li><li>test</li></li></p></li>');
        runTest(text, '<li><li><p><p>test</p></p></li></li>');
        runTest(text, '<li><li><p><li>test</li></p></li></li>');
        runTest(text, '<li><li><li><p>test</p></li></li></li>');
        runTest(text, '<li><li><li><li>test</li></li></li></li>');

        runTest(text, ' <li>test</li> <li>test</li> ');
        runTest(text, ' <p align="left">test</p> <p align="right">test</p> ');
        runTest(text, ' <p>test</p> <li>test</li> ');

        trace('==== various edge cases found by bruteforce testing');
        runTest(text, '<p align="right"><font color="#010101">text</font></p>');
        runTest(text, '<li><font color="#010101">text</font></li>');
        runTest(text, '<textformat leading="1">text<li></li></textformat>');

        trace('==== some real-world cases');
        runTest(text, '<TEXTFORMAT LEADING="1"><P ALIGN="LEFT"><FONT FACE="Noto Sans" SIZE="12" COLOR="#000000" LETTERSPACING="0" KERNING="0"></FONT></P></TEXTFORMAT><TEXTFORMAT LEADING="2"><P ALIGN="LEFT"><FONT FACE="Noto Sans" SIZE="12" COLOR="#000000" LETTERSPACING="0" KERNING="0"></FONT></P></TEXTFORMAT>\n');
        runTest(text, '<TEXTFORMAT LEADING="1"><P ALIGN="LEFT"><FONT FACE="Some Font" SIZE="10" COLOR="#000000" LETTERSPACING="0" KERNING="0"></FONT></P></TEXTFORMAT>\n');
        runTest(text, '<TEXTFORMAT LEADING="1"><P ALIGN="LEFT"><FONT FACE="Some Font" SIZE="10" COLOR="#000000" LETTERSPACING="0" KERNING="0"></FONT></P></TEXTFORMAT><font COLOR="#444444">some text\n\n</font>\n');

        // TODO AVM2 has a different behavior for
        //   mismatched tags compared to AVM1
        // trace('==== mismatched tags');
        // runTest(text, '<i>a<i>b</I>c</I>');
        // runTest(text, '<I>a<i>b</I>c</i>');
        // runTest(text, '<b>a<i>b</b>c</i>');
        // runTest(text, '<b>a<i>b</b>c</i>d</b>e');
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
