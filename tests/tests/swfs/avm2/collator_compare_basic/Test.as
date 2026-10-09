package {

import flash.display.Sprite;
import flash.globalization.Collator;
import flash.globalization.CollatorMode;

public class Test extends Sprite {
    public function Test() {
        testIdentity();
        testOrdering();
        testCaseAndDiacritics();
        testNull();
    }

    private function testIdentity():void {
        trace("// Identical strings");
        var collator:Collator = new Collator("en-US");
        for each (var s:String in ["", "a", "abc", "ABC", "é", "a b", "123"]) {
            trace("\"" + s + "\": " + collator.compare(s, s) + " " + collator.equals(s, s));
        }
    }

    private function testOrdering():void {
        trace("// Ordering");
        var collator:Collator = new Collator("en-US");
        var pairs:Array = [
            ["a", "b"],
            ["", "a"],
            ["app", "apple"],
            ["apple", "banana"],
            ["apple", "Banana"],
            ["Apple", "banana"],
            ["1", "2"],
            ["2", "a"],
            ["é", "z"],
            ["resume", "rf"],
            ["résumé", "rf"],
            ["a", "A"],
            ["e", "é"]
        ];
        for each (var pair:Array in pairs) {
            trace(pair[0] + " vs " + pair[1] + ": "
                + collator.compare(pair[0], pair[1]) + " " + collator.compare(pair[1], pair[0]) + " "
                + collator.equals(pair[0], pair[1]) + " " + collator.equals(pair[1], pair[0]));
        }
    }

    private function testCaseAndDiacritics():void {
        var pairs:Array = [
            ["abc", "ABC"],
            ["abc", "Abc"],
            ["e", "é"],
            ["resume", "RÉSUMÉ"],
            ["abc", "abd"]
        ];
        var configs:Array = [
            ["sorting", CollatorMode.SORTING, false, false],
            ["sorting, ignoreCase", CollatorMode.SORTING, true, false],
            ["sorting, ignoreDiacritics", CollatorMode.SORTING, false, true],
            ["sorting, ignoreCase, ignoreDiacritics", CollatorMode.SORTING, true, true],
            ["matching", CollatorMode.MATCHING, true, true]
        ];
        for each (var config:Array in configs) {
            trace("// " + config[0]);
            var collator:Collator = new Collator("en-US", config[1]);
            collator.ignoreCase = config[2];
            collator.ignoreDiacritics = config[3];
            for each (var pair:Array in pairs) {
                trace(pair[0] + " vs " + pair[1] + ": "
                    + collator.compare(pair[0], pair[1]) + " "
                    + collator.equals(pair[0], pair[1]));
            }
        }
    }

    private function testNull():void {
        trace("// null");
        var collator:Collator = new Collator("en-US");
        var pairs:Array = [[null, "a"], ["a", null], [null, null], [null, ""]];
        for each (var pair:Array in pairs) {
            var label:String = String(pair[0]) + " vs " + String(pair[1]);
            try {
                trace(label + " compare: " + collator.compare(pair[0], pair[1]));
            } catch (e:Error) {
                trace(label + " compare: " + Object(e).constructor);
                trace(e.getStackTrace());
            }
            try {
                trace(label + " equals: " + collator.equals(pair[0], pair[1]));
            } catch (e:Error) {
                trace(label + " equals: " + Object(e).constructor);
                trace(e.getStackTrace());
            }
        }
    }
}
}
