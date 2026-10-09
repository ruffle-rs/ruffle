package {

import flash.display.Sprite;
import flash.globalization.Collator;
import flash.globalization.CollatorMode;

public class Test extends Sprite {
    private static const FLAGS:Array = [
        "ignoreCase",
        "ignoreCharacterWidth",
        "ignoreDiacritics",
        "ignoreKanaType",
        "ignoreSymbols",
        "numericComparison"
    ];

    public function Test() {
        testDefaults();
        testSetting();
    }

    private function testDefaults():void {
        trace("// Defaults");
        var sorting:Collator = new Collator("en-US", CollatorMode.SORTING);
        trace("sorting: " + describe(sorting) + " " + sorting.lastOperationStatus);
        var matching:Collator = new Collator("en-US", CollatorMode.MATCHING);
        trace("matching: " + describe(matching) + " " + matching.lastOperationStatus);
    }

    private function testSetting():void {
        var values:Array = [true, false, null, 1, 0, "x", "", undefined];
        for each (var mode:String in [CollatorMode.SORTING, CollatorMode.MATCHING]) {
            for each (var flag:String in FLAGS) {
                trace("// " + flag + " (" + mode + ")");
                var collator:Collator = new Collator("en-US", mode);
                for each (var value:* in values) {
                    collator[flag] = value;
                    trace(String(value) + ": " + describe(collator));
                }
            }
        }
    }

    private function describe(collator:Collator):String {
        var result:Array = [];
        for each (var flag:String in FLAGS) {
            result.push(collator[flag]);
        }
        return result.join(",");
    }
}
}
