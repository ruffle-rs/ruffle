package {

import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    public function Test() {
        var ef:ElementFormat = new ElementFormat();
        trace(ef.alignmentBaseline);
        trace(ef.alpha);
        trace(ef.baselineShift);
        trace(ef.breakOpportunity);
        trace(ef.color);
        trace(ef.digitCase);
        trace(ef.digitWidth);
        trace(ef.dominantBaseline);
        trace(ef.fontDescription);
        trace(ef.fontSize);
        trace(ef.kerning);
        trace(ef.ligatureLevel);
        trace(ef.locale);
        trace(ef.locked);
        trace(ef.textRotation);
        trace(ef.trackingLeft);
        trace(ef.trackingRight);
        trace(ef.typographicCase);

        trace("alignmentBaseline");
        var getter = function() { return ef.alignmentBaseline; };
        testSetter(getter, function() { ef.alignmentBaseline = null; });
        testSetter(getter, function() { ef.alignmentBaseline = ""; });
        testSetter(getter, function() { ef.alignmentBaseline = "<invalid>"; });
        testSetter(getter, function() { ef.alignmentBaseline = "ascent"; });
        testSetter(getter, function() { ef.alignmentBaseline = "descent"; });
        testSetter(getter, function() { ef.alignmentBaseline = "ideographicBottom"; });
        testSetter(getter, function() { ef.alignmentBaseline = "ideographicCenter"; });
        testSetter(getter, function() { ef.alignmentBaseline = "ideographicTop"; });
        testSetter(getter, function() { ef.alignmentBaseline = "roman"; });
        testSetter(getter, function() { ef.alignmentBaseline = "useDominantBaseline"; });

        trace("alpha");
        var getter = function() { return ef.alpha; };
        testSetter(getter, function() { ef.alpha = null; });
        testSetter(getter, function() { ef.alpha = 0; });
        testSetter(getter, function() { ef.alpha = 0.7; });
        testSetter(getter, function() { ef.alpha = 1; });
        testSetter(getter, function() { ef.alpha = -1; });
        testSetter(getter, function() { ef.alpha = 5; });
        testSetter(getter, function() { ef.alpha = 5.2; });
        testSetter(getter, function() { ef.alpha = 0.0/0.0; });
        testSetter(getter, function() { ef.alpha = 1.0/0.0; });
        testSetter(getter, function() { ef.alpha = -1.0/0.0; });

        trace("baselineShift");
        var getter = function() { return ef.baselineShift; };
        testSetter(getter, function() { ef.baselineShift = null; });
        testSetter(getter, function() { ef.baselineShift = 0; });
        testSetter(getter, function() { ef.baselineShift = 0.7; });
        testSetter(getter, function() { ef.baselineShift = 1; });
        testSetter(getter, function() { ef.baselineShift = -1; });
        testSetter(getter, function() { ef.baselineShift = 5; });
        testSetter(getter, function() { ef.baselineShift = 5.2; });
        testSetter(getter, function() { ef.baselineShift = 0.0/0.0; });
        testSetter(getter, function() { ef.baselineShift = 1.0/0.0; });
        testSetter(getter, function() { ef.baselineShift = -1.0/0.0; });

        trace("breakOpportunity");
        var getter = function() { return ef.breakOpportunity; };
        testSetter(getter, function() { ef.breakOpportunity = null; });
        testSetter(getter, function() { ef.breakOpportunity = ""; });
        testSetter(getter, function() { ef.breakOpportunity = "<invalid>"; });
        testSetter(getter, function() { ef.breakOpportunity = "all"; });
        testSetter(getter, function() { ef.breakOpportunity = "any"; });
        testSetter(getter, function() { ef.breakOpportunity = "auto"; });
        testSetter(getter, function() { ef.breakOpportunity = "none"; });

        trace("color");
        var getter = function() { return ef.color; };
        testSetter(getter, function() { ef.color = null; });
        testSetter(getter, function() { ef.color = 0; });
        testSetter(getter, function() { ef.color = 0xFFFFFF; });
        testSetter(getter, function() { ef.color = 0xdeadbeef; });
        testSetter(getter, function() { ef.color = 0xFFFFFFFF; });

        trace("digitCase");
        var getter = function() { return ef.digitCase; };
        testSetter(getter, function() { ef.digitCase = null; });
        testSetter(getter, function() { ef.digitCase = ""; });
        testSetter(getter, function() { ef.digitCase = "<invalid>"; });
        testSetter(getter, function() { ef.digitCase = "default"; });
        testSetter(getter, function() { ef.digitCase = "lining"; });
        testSetter(getter, function() { ef.digitCase = "oldStyle"; });

        trace("digitWidth");
        var getter = function() { return ef.digitWidth; };
        testSetter(getter, function() { ef.digitWidth = null; });
        testSetter(getter, function() { ef.digitWidth = ""; });
        testSetter(getter, function() { ef.digitWidth = "<invalid>"; });
        testSetter(getter, function() { ef.digitWidth = "default"; });
        testSetter(getter, function() { ef.digitWidth = "proportional"; });
        testSetter(getter, function() { ef.digitWidth = "tabular"; });

        trace("dominantBaseline");
        var getter = function() { return ef.dominantBaseline; };
        testSetter(getter, function() { ef.dominantBaseline = null; });
        testSetter(getter, function() { ef.dominantBaseline = ""; });
        testSetter(getter, function() { ef.dominantBaseline = "<invalid>"; });
        testSetter(getter, function() { ef.dominantBaseline = "ascent"; });
        testSetter(getter, function() { ef.dominantBaseline = "descent"; });
        testSetter(getter, function() { ef.dominantBaseline = "ideographicBottom"; });
        testSetter(getter, function() { ef.dominantBaseline = "ideographicCenter"; });
        testSetter(getter, function() { ef.dominantBaseline = "ideographicTop"; });
        testSetter(getter, function() { ef.dominantBaseline = "roman"; });
        testSetter(getter, function() { ef.dominantBaseline = "useDominantBaseline"; });

        trace("fontDescription");
        var getter = function() { return ef.fontDescription; };
        testSetter(getter, function() { ef.fontDescription = null; });
        testSetter(getter, function() { ef.fontDescription = new FontDescription(); });

        trace("fontSize");
        var getter = function() { return ef.fontSize; };
        testSetter(getter, function() { ef.fontSize = null; });
        testSetter(getter, function() { ef.fontSize = 0; });
        testSetter(getter, function() { ef.fontSize = 0.7; });
        testSetter(getter, function() { ef.fontSize = 1; });
        testSetter(getter, function() { ef.fontSize = -1; });
        testSetter(getter, function() { ef.fontSize = 5; });
        testSetter(getter, function() { ef.fontSize = 5.2; });
        testSetter(getter, function() { ef.fontSize = 0.0/0.0; });
        testSetter(getter, function() { ef.fontSize = 1.0/0.0; });
        testSetter(getter, function() { ef.fontSize = -1.0/0.0; });

        trace("kerning");
        var getter = function() { return ef.kerning; };
        testSetter(getter, function() { ef.kerning = null; });
        testSetter(getter, function() { ef.kerning = ""; });
        testSetter(getter, function() { ef.kerning = "<invalid>"; });
        testSetter(getter, function() { ef.kerning = "on"; });
        testSetter(getter, function() { ef.kerning = "off"; });
        testSetter(getter, function() { ef.kerning = "auto"; });

        trace("ligatureLevel");
        var getter = function() { return ef.ligatureLevel; };
        testSetter(getter, function() { ef.ligatureLevel = null; });
        testSetter(getter, function() { ef.ligatureLevel = ""; });
        testSetter(getter, function() { ef.ligatureLevel = "<invalid>"; });
        testSetter(getter, function() { ef.ligatureLevel = "none"; });
        testSetter(getter, function() { ef.ligatureLevel = "minimum"; });
        testSetter(getter, function() { ef.ligatureLevel = "common"; });
        testSetter(getter, function() { ef.ligatureLevel = "uncommon"; });
        testSetter(getter, function() { ef.ligatureLevel = "exotic"; });

        trace("locale");
        var getter = function() { return ef.locale; };
        testSetter(getter, function() { ef.locale = null; });
        testSetter(getter, function() { ef.locale = ""; });
        testSetter(getter, function() { ef.locale = "<invalid>"; });
        testSetter(getter, function() { ef.locale = "!@#$%^&*()_+:\"{}\\"; });

        trace("locked");
        var getter = function() { return ef.locked; };
        testSetter(getter, function() { ef.locked = null; });
        testSetter(getter, function() { ef.locked = false; });
        testSetter(getter, function() { ef.locked = true; });

        // unlock EF
        ef = new ElementFormat();

        trace("textRotation");
        var getter = function() { return ef.textRotation; };
        testSetter(getter, function() { ef.textRotation = null; });
        testSetter(getter, function() { ef.textRotation = ""; });
        testSetter(getter, function() { ef.textRotation = "<invalid>"; });
        testSetter(getter, function() { ef.textRotation = "auto"; });
        testSetter(getter, function() { ef.textRotation = "rotate0"; });
        testSetter(getter, function() { ef.textRotation = "rotate90"; });
        testSetter(getter, function() { ef.textRotation = "rotate180"; });
        testSetter(getter, function() { ef.textRotation = "rotate270"; });

        trace("trackingLeft");
        var getter = function() { return ef.trackingLeft; };
        testSetter(getter, function() { ef.trackingLeft = null; });
        testSetter(getter, function() { ef.trackingLeft = 0; });
        testSetter(getter, function() { ef.trackingLeft = 0.7; });
        testSetter(getter, function() { ef.trackingLeft = 1; });
        testSetter(getter, function() { ef.trackingLeft = -1; });
        testSetter(getter, function() { ef.trackingLeft = 5; });
        testSetter(getter, function() { ef.trackingLeft = 5.2; });
        testSetter(getter, function() { ef.trackingLeft = 0.0/0.0; });
        testSetter(getter, function() { ef.trackingLeft = 1.0/0.0; });
        testSetter(getter, function() { ef.trackingLeft = -1.0/0.0; });

        trace("trackingRight");
        var getter = function() { return ef.trackingRight; };
        testSetter(getter, function() { ef.trackingRight = null; });
        testSetter(getter, function() { ef.trackingRight = 0; });
        testSetter(getter, function() { ef.trackingRight = 0.7; });
        testSetter(getter, function() { ef.trackingRight = 1; });
        testSetter(getter, function() { ef.trackingRight = -1; });
        testSetter(getter, function() { ef.trackingRight = 5; });
        testSetter(getter, function() { ef.trackingRight = 5.2; });
        testSetter(getter, function() { ef.trackingRight = 0.0/0.0; });
        testSetter(getter, function() { ef.trackingRight = 1.0/0.0; });
        testSetter(getter, function() { ef.trackingRight = -1.0/0.0; });

        trace("typographicCase");
        var getter = function() { return ef.typographicCase; };
        testSetter(getter, function() { ef.typographicCase = null; });
        testSetter(getter, function() { ef.typographicCase = ""; });
        testSetter(getter, function() { ef.typographicCase = "<invalid>"; });
        testSetter(getter, function() { ef.typographicCase = "caps"; });
        testSetter(getter, function() { ef.typographicCase = "capsAndSmallCaps"; });
        testSetter(getter, function() { ef.typographicCase = "default"; });
        testSetter(getter, function() { ef.typographicCase = "lowercase"; });
        testSetter(getter, function() { ef.typographicCase = "smallCaps"; });
        testSetter(getter, function() { ef.typographicCase = "title"; });
        testSetter(getter, function() { ef.typographicCase = "uppercase"; });
    }

    private function testSetter(getter:Function, setter:Function) {
        try {
            setter();
        } catch (e) {
            trace("  Caught error:" + e.getStackTrace().split("\n").slice(0, 2).join("\n"));
        }

        trace("  Value:" + getter());
    }
}

}
