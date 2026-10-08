package {

import flash.display.Sprite;
import flash.globalization.Collator;
import flash.globalization.CollatorMode;
import flash.globalization.LocaleID;

public class Test extends Sprite {
    public function Test() {
        testGetAvailableLocaleIDNames();
        testRequestedLocaleIDName();
    }

    private function testGetAvailableLocaleIDNames():void {
        var names:Vector.<String> = Collator.getAvailableLocaleIDNames();

        trace("// getAvailableLocaleIDNames()");
        trace("is Vector.<String>: " + (names is Vector.<String>));
        trace("fixed: " + names.fixed);
        trace("non-empty: " + (names.length > 0));

        var allNonEmpty:Boolean = true;
        for each (var name:String in names) {
            if (name == null || name.length == 0) {
                allNonEmpty = false;
            }
        }
        trace("all entries non-empty: " + allNonEmpty);

        trace("// Calling getAvailableLocaleIDNames() again");
        var other:Vector.<String> = Collator.getAvailableLocaleIDNames();
        trace("same object: " + (other === names));
        trace("same length: " + (other.length == names.length));

        trace("// Modifying the returned vector");
        names.push("__test__");
        names[0] = "__test0__";
        var fresh:Vector.<String> = Collator.getAvailableLocaleIDNames();
        trace("contains pushed entry: " + (fresh.indexOf("__test__") >= 0));
        trace("contains replaced entry: " + (fresh.indexOf("__test0__") >= 0));
    }

    private function testRequestedLocaleIDName():void {
        trace("// requestedLocaleIDName");
        var ids:Array = [
            "en-US",
            "en",
            "fr-FR",
            "zh-Hans-CN",
            "sr-Latn-RS",
            "en-US-POSIX",
            "es-419",
            "en-001",
            "en-XX",
            "de-DE@collation=phonebook",
            LocaleID.DEFAULT
        ];
        for each (var id:String in ids) {
            testRequestedLocaleIDNameFor(id, CollatorMode.SORTING);
        }

        trace("// requestedLocaleIDName with CollatorMode.MATCHING");
        testRequestedLocaleIDNameFor("en-US", CollatorMode.MATCHING);

        trace("// Invalid initialMode");
        var modes:Array = [
            "invalid",
            "",
            "SORTING",
            "Sorting",
            "MATCHING",
            " sorting",
            "sorting ",
            null
        ];
        for each (var mode:String in modes) {
            testRequestedLocaleIDNameFor("en-US", mode);
        }

        trace("// Null requestedLocaleIDName");
        testRequestedLocaleIDNameFor(null, CollatorMode.SORTING);
        testRequestedLocaleIDNameFor(null, "invalid");
        testRequestedLocaleIDNameFor(null, null);

        trace("// Empty requestedLocaleIDName");
        testRequestedLocaleIDNameFor("", CollatorMode.SORTING);
        testRequestedLocaleIDNameFor("", "invalid");
        testRequestedLocaleIDNameFor("", null);
    }

    private function testRequestedLocaleIDNameFor(id:String, mode:String):void {
        try {
            var collator:Collator = new Collator(id, mode);
            trace(describe(id) + ", " + describe(mode) + ": " + describe(collator.requestedLocaleIDName));
        } catch (e:Error) {
            trace(describe(id) + ", " + describe(mode) + ": " + Object(e).constructor);
            trace(e.getStackTrace());
        }
    }

    private function describe(value:String):String {
        return value === null ? "null" : "\"" + value + "\"";
    }
}
}
