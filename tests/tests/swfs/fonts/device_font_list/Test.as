package {
import flash.display.*;
import flash.text.*;

[SWF(width="100", height="200")]
public class Test extends Sprite {
    [Embed(source="TestFontA.ttf", fontName="EmbeddedTestFontA", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var EmbeddedTestFontA:Class;

    [Embed(source="TestFontB.ttf", fontName="EmbeddedTestFontB", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var EmbeddedTestFontB:Class;

    private var nextY: Number = 0;

    private var fontListsDevice: Array = [
        "Totally Unknown, TestFontA , TestFontB",
        " testFOntB  , TestFontA , TestFontB",
    ];
    private var fontListsEmbedded: Array = [
        "Totally Unknown, EmbeddedTestFontA, EmbeddedTestFontB",
        "EmbeddedTestFontA",
        "  EmbeddedTestFontA",
        "EmbeddedTestFontA  ",
        "  EmbeddedTestFontA  ",
        "  embeddedTESTFonta",
    ];

    public function Test() {
        stage.scaleMode = "noScale";

        for each (var embedded in [false, true]) {
            var fontLists = embedded ? fontListsEmbedded : fontListsDevice;
            for each (var fontList in fontLists) {
                testFontListCss(embedded, fontList);
                testFontListFormat(embedded, fontList);
            }
        }
    }

    function testFontListCss(embedded: Boolean, fontList: String) {
        trace("Testing CSS font list fallback:");
        trace("  Embedded? = " + embedded);
        trace("  Font list? = " + fontList);

        var style: StyleSheet = new StyleSheet();

        var classFontList:Object = new Object();
        classFontList.fontFamily = fontList;
        classFontList.fontSize = 20;
        style.setStyle(".fontlist", classFontList);

        var text: TextField = new TextField();
        text.embedFonts = embedded;
        text.styleSheet = style;

        text.width = 100;
        text.height = 50;
        text.y = nextY;
        nextY += text.height;
        text.text = "<span class='fontlist'>abc</span>";

        addChild(text);

        traceChars(text);
    }

    function testFontListFormat(embedded: Boolean, fontList: String) {
        trace("Testing TextFormat font list fallback:");
        trace("  Embedded? = " + embedded);
        trace("  Font list? = " + fontList);

        var tf: TextFormat = new TextFormat(fontList, 20);
        var text: TextField = new TextField();
        text.embedFonts = embedded;
        text.defaultTextFormat = tf;

        text.width = 100;
        text.height = 50;
        text.y = nextY;
        nextY += text.height;
        text.text = "abc";

        addChild(text);

        traceChars(text);
    }

    private function traceChars(text: TextField) {
        traceChar(text, 0);
        traceChar(text, 1);
        traceChar(text, 2);
    }

    private function traceChar(text: TextField, i: int) {
        try {
            var ch: Number = text.getCharBoundaries(i).width;
            if (ch == 32) {
                trace("  Char " + i + " is TestFontA");
            }
            if (ch == 30) {
                trace("  Char " + i + " is TestFontB");
            }
        } catch(e) {}
    }
}
}
