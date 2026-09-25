package {

import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    public function Test() {
        var o:TabStop = new TabStop();
        trace(o.alignment);
        trace(o.position);
        trace(o.decimalAlignmentToken);

        trace("constructor");
        trace(new TabStop().alignment);
        trace(new TabStop().position);
        trace(new TabStop().decimalAlignmentToken);
        trace(new TabStop(TabAlignment.CENTER).alignment);
        trace(new TabStop(TabAlignment.CENTER, 12).alignment);
        trace(new TabStop(TabAlignment.CENTER, 12).position);
        trace(new TabStop(TabAlignment.CENTER, 12, ".").alignment);
        trace(new TabStop(TabAlignment.CENTER, 12, ".").position);
        trace(new TabStop(TabAlignment.CENTER, 12, ".").decimalAlignmentToken);

        testConstructor(function() { new TabStop("invalid", -20, null); });
        testConstructor(function() { new TabStop("start", -20, null); });
        testConstructor(function() { new TabStop("start", 2, null); });
        testConstructor(function() { new TabStop("start", 2, "5"); });

        trace("alignment");
        var getter = function() { return o.alignment; };
        testSetter(getter, function() { o.alignment = null; });
        testSetter(getter, function() { o.alignment = ""; });
        testSetter(getter, function() { o.alignment = "<invalid>"; });
        testSetter(getter, function() { o.alignment = "start"; });
        testSetter(getter, function() { o.alignment = "Start"; });
        testSetter(getter, function() { o.alignment = "center"; });
        testSetter(getter, function() { o.alignment = "end"; });
        testSetter(getter, function() { o.alignment = "decimal"; });
        testSetter(getter, function() { o.alignment = TabAlignment.CENTER; });
        testSetter(getter, function() { o.alignment = TabAlignment.DECIMAL; });
        testSetter(getter, function() { o.alignment = TabAlignment.END; });
        testSetter(getter, function() { o.alignment = TabAlignment.START; });

        trace("position");
        var getter = function() { return o.position; };
        testSetter(getter, function() { o.position = null; });
        testSetter(getter, function() { o.position = 0; });
        testSetter(getter, function() { o.position = 0.1; });
        testSetter(getter, function() { o.position = 0.5; });
        testSetter(getter, function() { o.position = 0.9; });
        testSetter(getter, function() { o.position = 1.1; });
        testSetter(getter, function() { o.position = 100; });
        testSetter(getter, function() { o.position = 100000; });
        testSetter(getter, function() { o.position = -2; });
        testSetter(getter, function() { o.position = -26.8; });
        testSetter(getter, function() { o.position = 0.0/0.0; });
        testSetter(getter, function() { o.position = 1.0/0.0; });
        testSetter(getter, function() { o.position = -1.0/0.0; });

        trace("decimalAlignmentToken");
        var getter = function() { return o.decimalAlignmentToken; };
        testSetter(getter, function() { o.decimalAlignmentToken = null; });
        testSetter(getter, function() { o.decimalAlignmentToken = ""; });
        testSetter(getter, function() { o.decimalAlignmentToken = "."; });
        testSetter(getter, function() { o.decimalAlignmentToken = ","; });
        testSetter(getter, function() { o.decimalAlignmentToken = "abc"; });
    }

    private function testConstructor(c:Function) {
        try {
            c();
            trace("  Constructed properly");
        } catch (e) {
            trace("  Caught error:" + e.getStackTrace());
        }
    }

    private function testSetter(getter:Function, setter:Function) {
        try {
            setter();
        } catch (e) {
            trace("  Caught error:" + e.getStackTrace());
        }

        trace("  Value:" + getter());
    }
}

}
