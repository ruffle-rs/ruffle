package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    private var nextY:int = 0;

    public function Test() {
        runDefaultCase("A<img src='missing.jpg'>B");
        runDefaultCase("A<img src='missing.jpg' id='hello' width='40.9' height='30.7' align='right' hspace='4.9' vspace='6.2'>B");
        runDefaultCase("A<img src='missing.jpg' align='left'>B");
        runDefaultCase("A<img src='missing.jpg' align='middle'>B");
        runDefaultCase("A<img id='hello' width='20' height='30'>B");
        runDefaultCase("A<img>B");
        runDefaultCase("A<img src=''>B");
        runDefaultCase("A<img src='missing.jpg' />B");
        runDefaultCase("A<img src='foo&amp;bar.jpg' id='a&amp;b'>B");
        runDefaultCase("A<img src='missing.jpg' width='abc' height='xyz'>B");
        runDefaultCase("A<img src='one.jpg'>B<img src='two.jpg'>C");
        runDefaultCase("A<img src='one.jpg' id='a'>B<img src='two.jpg' id='a'>C");
        runDefaultCase("A   <img src='missing.jpg'>   B");
        runDefaultCase("A<img src='missing.jpg' align='LEFT'>B");
        runDefaultCase("A<img src='missing.jpg' align='Left'>B");
        runDefaultCase("A<img src='missing.jpg' align='RIGHT'>B");
        runDefaultCase("A<img src='missing.jpg' align='Right'>B");
        runDefaultCase("A<img src='missing.jpg' align='something'>B");
        runDefaultCase("A<img src='missing.jpg' checkPolicyFile='true' unknown='value'>B");
        runDefaultCase("<font size='2'>A</font><img src='missing.jpg'>B");
        runDefaultCase("A<img src='missing.jpg'><font size='2'>B</font>");
        runDefaultCase("<font size='20'>A<img src='missing.jpg'>B</font>");
        runDefaultCase("<b>A<img src='missing.jpg'>B</b>");
        runDefaultCase("<i>A<img src='missing.jpg'>B</i>");
        runDefaultCase("<u>A<img src='missing.jpg'>B</u>");
        runDefaultCase("A<img src='missing.jpg'>X</img>B");
        runDefaultCase("A<img src='\".jpg' id='\"'>B");
        runDefaultCase("A<img src='missing.jpg' checkPolicyFile='True'>B");
        runDefaultCase("A<img src='missing.jpg' checkPolicyFile='false'>B");
        runDefaultCase("A<img src='missing.jpg' checkPolicyFile='FALSE'>B");
        runDefaultCase("A<img src='missing.jpg' checkPolicyFile='unknown'>B");
        runDefaultCase("<img src='a.jpg'><img src='b.jpg'>");
        runDefaultCase("A<img src='a.jpg'><img src='a.jpg'>B");
        runDefaultCase("A<img src='a.jpg' id='x'><img src='a.jpg'>B");
        runDefaultCase("A<img src='a.jpg' width='7'><img src='a.jpg'>B");
        runDefaultCase("A<img src='a.jpg' height='7'><img src='a.jpg'>B");
        runDefaultCase("A<img src='a.jpg' height='7'><img src='a.jpg' height='7'>B");
        runDefaultCase("A<img src='a.jpg' align='right'><img src='a.jpg'>B");
        runDefaultCase("A<img src='a.jpg' hspace='7'><img src='a.jpg'>B");
        runDefaultCase("A<img src='a.jpg' hspace='7'><img src='a.jpg' hspace='7'>B");
        runDefaultCase("A<img src='a.jpg' vspace='7'><img src='a.jpg'>B");
        runDefaultCase("A<img src='a.jpg' vspace='7'><img src='a.jpg' vspace='7'>B");
        runDefaultCase("A<img src='a.jpg' checkPolicyFile='true'><img src='a.jpg'>B");
        runDefaultCase("A<img src='a.jpg' checkPolicyFile='true'><img src='a.jpg' checkPolicyFile='true'>B");
        runDefaultCase("<img src='m.jpg'>");
        runDefaultCase("<img src='m.jpg'>B");
        runDefaultCase("A<img src='m.jpg'>");
        runDefaultCase("<a href='http://x'>A<img src='m.jpg'>B</a>");
        runDefaultCase("A<b><img src='m.jpg'></b>B");
        runDefaultCase("A<i><img src='m.jpg'></i>B");
        runDefaultCase("A<u><img src='m.jpg'></u>B");
        runDefaultCase("<p align='center'>A<img src='m.jpg'>B</p>");
        runDefaultCase("<li>A<img src='m.jpg'>B</li>");
        runDefaultCase("<textformat indent='5'>A<img src='m.jpg'>B</textformat>");
        runDefaultCase("<font face='Arial' color='#FF0000'>A<img src='m.jpg'>B</font>");
        runDefaultCase("A<br><img src='m.jpg'><br>B");
        runDefaultCase("<p>A</p><img src='m.jpg'><p>B</p>");
        runDefaultCase("A<IMG SRC='m.jpg' WIDTH='10' ALIGN='right'>B");
        runDefaultCase("A<Img Src='m.jpg'>B");
        runDefaultCase("A<img src='a.jpg' src='b.jpg' id='x' id='y'>B");
        runDefaultCase("A<img src=\"m.jpg\" id=\"q\">B");
        runDefaultCase("A<img src=' m.jpg ' id=' x '>B");
        runDefaultCase("A<img src='m.jpg' align=' right'>B");
        runDefaultCase("A<img src='m.jpg' align=''>B");
        runDefaultCase("A<img src='m.jpg' id=''>B");
        runDefaultCase("A<img src=' '>B");
        runDefaultCase("A<img src='m.jpg' checkPolicyFile=''>B");

        runAllCases("A   <img src='missing.jpg'>   B");
        runAllCases("A\n  <img src='missing.jpg'>\n  B");
        runAllCases("A  \n<img src='missing.jpg'>\n  B");
        runAllCases("A  \n<img src='missing.jpg'>  \nB");

        runReparseCase("A<img src='missing.jpg'>B");
    }

    private function runDefaultCase(html:String):void {
        runCase(html, true, false);
    }

    private function runAllCases(html:String):void {
        runCase(html, true, true);
        runCase(html, true, false);
        runCase(html, false, true);
        runCase(html, false, false);
    }

    private function runCase(
        html:String,
        multiline:Boolean,
        condenseWhite:Boolean
    ):void {
        var tf:TextField = createTextField();

        tf.multiline = multiline;
        tf.condenseWhite = condenseWhite;
        tf.htmlText = html;

        trace("SOURCE=" + escapeText(html));
        trace("MULTILINE=" + multiline);
        trace("CONDENSE_WHITE=" + condenseWhite);
        trace("TEXT=" + escapeText(tf.text));
        trace("HTML=" + tf.htmlText);
        trace("---");
    }

    private function runReparseCase(html:String):void {
        var tf:TextField = createTextField();

        tf.multiline = true;
        tf.condenseWhite = false;
        tf.htmlText = html;

        var firstHtml:String = tf.htmlText;

        trace("REPARSE_SOURCE=[" + escapeText(html) + "]");
        trace("FIRST_TEXT=[" + escapeText(tf.text) + "]");
        trace("FIRST_HTML=[" + firstHtml + "]");

        tf.htmlText = firstHtml + " ";

        trace("SECOND_SOURCE=[" + escapeText(firstHtml + " ") + "]");
        trace("SECOND_TEXT=[" + escapeText(tf.text) + "]");
        trace("SECOND_HTML=[" + tf.htmlText + "]");
        trace("---");
    }

    private function createTextField():TextField {
        var tf:TextField = new TextField();
        tf.width = 400;
        tf.height = 300;
        tf.y = nextY++;

        var format:TextFormat = new TextFormat();
        format.font = "Times";
        format.size = 12;
        tf.defaultTextFormat = format;

        addChild(tf);
        return tf;
    }

    private function escapeText(value:String):String {
        return value
            .replace(/\r/g, "\\r")
            .replace(/\n/g, "\\n")
            .replace(/Times New Roman/g, "Times");
    }
}
}
