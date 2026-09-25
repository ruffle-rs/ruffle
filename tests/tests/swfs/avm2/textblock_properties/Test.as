package {

import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    public function Test() {
        var o:TextBlock = new TextBlock();
        trace(o.applyNonLinearFontScaling);
        trace(o.baselineFontDescription);
        trace(o.baselineFontSize);
        trace(o.baselineZero);
        trace(o.bidiLevel);
        trace(o.content);
        trace(o.firstInvalidLine);
        trace(o.firstLine);
        trace(o.lastLine);
        trace(o.lineRotation);
        trace(o.tabStops);
        trace(o.textJustifier);
        trace(o.textLineCreationResult);

        trace("applyNonLinearFontScaling");
        var getter = function() { return o.applyNonLinearFontScaling; };
        testSetter(getter, function() { o.applyNonLinearFontScaling = true; });
        testSetter(getter, function() { o.applyNonLinearFontScaling = false; });

        trace("baselineFontDescription");
        var getter = function() { return o.baselineFontDescription; };
        testSetter(getter, function() { o.baselineFontDescription = null; });
        testSetter(getter, function() { o.baselineFontDescription = new FontDescription(); });

        trace("baselineFontSize");
        var getter = function() { return o.baselineFontSize; };
        testSetter(getter, function() { o.baselineFontSize = null; });
        testSetter(getter, function() { o.baselineFontSize = 0.0; });
        testSetter(getter, function() { o.baselineFontSize = 0.1; });
        testSetter(getter, function() { o.baselineFontSize = 0.5; });
        testSetter(getter, function() { o.baselineFontSize = 0.9; });
        testSetter(getter, function() { o.baselineFontSize = 1.1; });
        testSetter(getter, function() { o.baselineFontSize = 1.5; });
        testSetter(getter, function() { o.baselineFontSize = 100; });
        testSetter(getter, function() { o.baselineFontSize = 100000; });
        testSetter(getter, function() { o.baselineFontSize = -2; });
        testSetter(getter, function() { o.baselineFontSize = -26.8; });
        testSetter(getter, function() { o.baselineFontSize = 0.0/0.0; });
        testSetter(getter, function() { o.baselineFontSize = 1.0/0.0; });
        testSetter(getter, function() { o.baselineFontSize = -1.0/0.0; });

        trace("baselineZero");
        var getter = function() { return o.baselineZero; };
        testSetter(getter, function() { o.baselineZero = null; });
        testSetter(getter, function() { o.baselineZero = ""; });
        testSetter(getter, function() { o.baselineZero = "<invalid>"; });
        testSetter(getter, function() { o.baselineZero = "ascent"; });
        testSetter(getter, function() { o.baselineZero = "Ascent"; });
        testSetter(getter, function() { o.baselineZero = "descent"; });
        testSetter(getter, function() { o.baselineZero = "ideographicBottom"; });
        testSetter(getter, function() { o.baselineZero = "ideographicCenter"; });
        testSetter(getter, function() { o.baselineZero = "ideographicTop"; });
        testSetter(getter, function() { o.baselineZero = "roman"; });
        testSetter(getter, function() { o.baselineZero = "useDominantBaseline"; });

        trace("bidiLevel");
        var getter = function() { return o.bidiLevel; };
        testSetter(getter, function() { o.bidiLevel = null; });
        testSetter(getter, function() { o.bidiLevel = 0; });
        testSetter(getter, function() { o.bidiLevel = 1; });
        testSetter(getter, function() { o.bidiLevel = 2; });
        testSetter(getter, function() { o.bidiLevel = 5; });
        testSetter(getter, function() { o.bidiLevel = 50; });
        testSetter(getter, function() { o.bidiLevel = 5000; });
        testSetter(getter, function() { o.bidiLevel = -1; });
        testSetter(getter, function() { o.bidiLevel = -10; });

        trace("content");
        var getter = function() { return o.content; };
        testSetter(getter, function() { o.content = null; });
        testSetter(getter, function() { o.content = new TextElement(); });

        trace("lineRotation");
        var getter = function() { return o.lineRotation; };
        testSetter(getter, function() { o.lineRotation = null; });
        testSetter(getter, function() { o.lineRotation = ""; });
        testSetter(getter, function() { o.lineRotation = "<invalid>"; });
        testSetter(getter, function() { o.lineRotation = "auto"; });
        testSetter(getter, function() { o.lineRotation = "AUTO"; });
        testSetter(getter, function() { o.lineRotation = "rotate0"; });
        testSetter(getter, function() { o.lineRotation = "Rotate0"; });
        testSetter(getter, function() { o.lineRotation = "rotate90"; });
        testSetter(getter, function() { o.lineRotation = "rotate180"; });
        testSetter(getter, function() { o.lineRotation = "rotate270"; });

        trace("tabStops");
        var getter = function() { return o.tabStops; };
        testSetter(getter, function() { o.tabStops = null; });
        testSetter(getter, function() { o.tabStops = new Vector.<TabStop>(); });

        trace("textJustifier");
        var getter = function() { return o.textJustifier; };
        testSetter(getter, function() { o.textJustifier = null; });
        testSetter(getter, function() { o.textJustifier = new SpaceJustifier(); });

        trace("tabStops mutability");
        var ts:Vector.<TabStop> = new Vector.<TabStop>();
        var ts0:TabStop = new TabStop();
        ts.push(ts0);
        o.tabStops = ts;
        trace(o.tabStops.length);
        ts.push(new TabStop());
        trace(o.tabStops.length);
        o.tabStops.push(new TabStop());
        trace(o.tabStops.length);
        trace(o.tabStops[0].position);
        ts0.position = 2;
        trace(o.tabStops[0].position);
        o.tabStops[0].position = 3;
        trace(o.tabStops[0].position);
        var ts1:TabStop = o.tabStops[0];
        o.tabStops[0].position = 4;
        trace(ts1.position);
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
