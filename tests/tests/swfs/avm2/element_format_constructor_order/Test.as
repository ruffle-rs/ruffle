package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;

public class Test extends Sprite {
    function Test() {
        test();
        test("invalid");
        test("invalid", "invalid");
        test("invalid", "invalid", "invalid");
        test("invalid", "invalid", "invalid", "invalid");
        test("invalid", "invalid", "invalid", "invalid", "invalid");
        test("invalid", "invalid", "invalid", "invalid", "invalid", "invalid");
        test("invalid", "invalid", "invalid", "invalid", "invalid", "invalid", "invalid");
        test("invalid", "invalid", "invalid", "invalid", "invalid", "invalid", "invalid", "invalid");
        test("invalid", "invalid", "invalid", "invalid", "invalid", "invalid", "invalid", "invalid", "invalid");
    }

    private function test(typographicCase:String = "default",
                          ligatureLevel:String = "common",
                          digitWidth:String = "default",
                          digitCase:String = "default",
                          breakOpportunity:String = "auto",
                          kerning:String = "on",
                          alignmentBaseline:String = "useDominantBaseline",
                          dominantBaseline:String = "roman",
                          textRotation:String = "auto"): void {
        trace("textRotation: " + textRotation + " " +
              "dominantBaseline: " + dominantBaseline + " " +
              "alignmentBaseline: " + alignmentBaseline + " " +
              "kerning: " + kerning + " " +
              "breakOpportunity: " + breakOpportunity + " " +
              "digitCase: " + digitCase + " " +
              "digitWidth: " + digitWidth + " " +
              "ligatureLevel: " + ligatureLevel + " " +
              "typographicCase: " + typographicCase)

        try {
            new ElementFormat(
            /* fontDescription */ null,
            /* fontSize */ 12,
            /* color */ 0,
            /* alpha */ 1,
            textRotation,
            dominantBaseline,
            alignmentBaseline,
            /* baselineShift */ 0,
            kerning,
            /* trackingRight */ 0,
            /* trackingLeft */ 0,
            /* locale */ "en",
            breakOpportunity,
            digitCase,
            digitWidth,
            ligatureLevel,
            typographicCase);
        } catch (e:*) {
            trace(e.getStackTrace());
        }

        trace("");
    }
}

}
