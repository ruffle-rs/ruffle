package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    private var nextY:int = 0;

    public function Test() {
        runDefaultCase(
            "A<img src='missing.jpg'>B"
        );

        runDefaultCase(
            "A<img src='missing.jpg' id='hello' width='40.9' height='30.7' align='right' hspace='4.9' vspace='6.2'>B"
        );

        runDefaultCase(
            "A<img src='missing.jpg' align='left'>B"
        );

        runDefaultCase(
            "A<img src='missing.jpg' align='middle'>B"
        );

        runDefaultCase(
            "A<img id='hello' width='20' height='30'>B"
        );

        runDefaultCase(
            "A<img>B"
        );

        runDefaultCase(
            "A<img src=''>B"
        );

        runDefaultCase(
            "A<img src='missing.jpg' />B"
        );

        runDefaultCase(
            "A<img src='foo&amp;bar.jpg' id='a&amp;b'>B"
        );

        runDefaultCase(
            "A<img src='missing.jpg' width='abc' height='xyz'>B"
        );

        runDefaultCase(
            "A<img src='one.jpg'>B<img src='two.jpg'>C"
        );

        runDefaultCase(
            "A   <img src='missing.jpg'>   B"
        );

        runCase(
            "A   <img src='missing.jpg'>   B",
            false,
            false
        );

        runCase(
            "A   <img src='missing.jpg'>   B",
            true,
            true
        );

        runCase(
            "A   <img src='missing.jpg'>   B",
            false,
            true
        );

        runCase(
            "A\n  <img src='missing.jpg'>\n  B",
            true,
            false
        );

        runCase(
            "A\n  <img src='missing.jpg'>\n  B",
            true,
            true
        );

        runDefaultCase(
            "A<img src='missing.jpg' width='40.9px' height='30.7xyz' hspace='4.9px' vspace='6.2xyz'>B"
        );

        runDefaultCase(
            "A<img src='missing.jpg' align='LEFT'>B"
        );

        runDefaultCase(
            "A<img src='missing.jpg' align='Left'>B"
        );

        runDefaultCase(
            "A<img src='missing.jpg' align='something'>B"
        );

        runDefaultCase(
            "A<img src='missing.jpg' checkPolicyFile='true' unknown='value'>B"
        );

        runReparseCase(
            "A<img src='missing.jpg'>B"
        );
    }

    private function runDefaultCase(html:String):void {
        runCase(html, true, false);
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
