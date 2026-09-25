package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            var obj = {
                "toJSON": function() {
                    return {e: "test"};
                }
            };

            var test = {
                "a": "b",
                "c": 2,
                "d": [1, 2, 3],
                "e": {
                    "f": undefined,
                    "g": null,
                    "h": "i"
                },
                "j": obj,
                "k": 5.3
            };

            get_props(JSON.stringify(test));

            get_props(JSON.stringify(test, function(k, v) {
                if (v == "b" || v == "i") {
                    return "replacement";
                }
                return v;
            }));

            get_props(JSON.stringify(test, null, 1));

            get_props(JSON.stringify(test, null, 20));

            trace(JSON.stringify(test, null, "custom").length);
            trace(JSON.stringify(test, ["a", "e", "f"]).length);

            var sealed = new SealedClass("Hello", true);

            // Flash can return the properties in any order, with the sole
            // exception that the getter must appear at the end. So we allow
            // any valid order to pass the test.
            var stringifiedSealed:String = JSON.stringify(sealed);
            stringifiedSealed = stringifiedSealed.replace('"prop1":"Hello"', '1');
            stringifiedSealed = stringifiedSealed.replace('"prop2":true', '2');
            stringifiedSealed = stringifiedSealed.replace('"MY_CONST":"Const val"', '3');
            stringifiedSealed = stringifiedSealed.replace('"myGetter":"Getter value"', 'g');
            if (stringifiedSealed === "{1,2,3,g}" || stringifiedSealed === "{1,3,2,g}" || stringifiedSealed === "{2,1,3,g}" || stringifiedSealed === "{2,3,1,g}" || stringifiedSealed === "{3,1,2,g}" || stringifiedSealed === "{3,2,1,g}") {
                trace("Stringification of SealedClass was correct");
            } else {
                trace("Stringification of SealedClass was incorrect! (got " + stringifiedSealed + ")");
            }

            var dynamicObj = new DynamicClass("Dynamic", false);
            dynamicObj["dyn1"] = "Dyn prop";
            dynamicObj["dyn2"] = 25;

            dynamicObj.setPropertyIsEnumerable("dyn1", true);
            dynamicObj.setPropertyIsEnumerable("dyn2", false);

            // Flash can return the properties in any order. So we allow any
            // order to pass the test, as long as all properties were added.
            var stringifiedDynamic:String = JSON.stringify(dynamicObj);
            stringifiedDynamic = stringifiedDynamic.replace('"prop1":"Dynamic"', '1');
            stringifiedDynamic = stringifiedDynamic.replace('"prop2":false', '2');
            stringifiedDynamic = stringifiedDynamic.replace('"dyn1":"Dyn prop"', '3');
            if (stringifiedDynamic === "{1,2,3}" || stringifiedDynamic === "{1,3,2}" || stringifiedDynamic === "{2,1,3}" || stringifiedDynamic === "{2,3,1}" || stringifiedDynamic === "{3,1,2}" || stringifiedDynamic === "{3,2,1}") {
                trace("Stringification of DynamicClass was correct");
            } else {
                trace("Stringification of DynamicClass was incorrect! (got " + stringifiedDynamic + ")");
            }
        }

        public static function get_props(str:String):void {
            var parsed = JSON.parse(str);
            with (parsed) {
                trace(a, c, d, e, e.f, e.g, e.h, j, j.e, k);
            }
            trace(str.length);
        }
    }
}

class SealedClass {
    public var prop1: String;
    internal var privateProp: String = "Hidden";
    public var prop2: Boolean;
    public const MY_CONST: String = "Const val";

    function SealedClass(prop1: String, prop2: Boolean) {
        this.prop1 = prop1;
        this.prop2 = prop2;
    }

    public function get myGetter():String {
        return "Getter value";
    }
}

dynamic class DynamicClass {
    public var prop1: String;
    internal var privateProp: String = "Hidden";
    public var prop2: Boolean;

    function DynamicClass(prop1: String, prop2: Boolean) {
        this.prop1 = prop1;
        this.prop2 = prop2;
    }
}
