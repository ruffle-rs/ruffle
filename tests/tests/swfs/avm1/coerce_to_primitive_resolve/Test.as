// Compile with:
//  mtasc -main -header 200:200:30 -version 8 Test.as -swf test.swf
class Test {
    
    static function main(current) {
        trace("// obj1: methods as members");
        var obj1 = {
            __proto__: null,
            valueOf: function() {
                trace("valueOf called!");
                return 1;
            },
            toString: function() {
                trace("toString called!");
                return "[obj as string]";
            }
        };

        trace("obj1 as num: " + (0 + obj1));
        trace(obj1);
        trace("");

        trace("// obj2: methods as properties");
        var obj2 = { __proto__: null };
        Object.prototype.addProperty.call(obj2, "valueOf", function() {
            trace("valueOf property called!");
            return obj1.valueOf;
        }, null);
        Object.prototype.addProperty.call(obj2, "toString", function() {
            trace("toString property called!");
            return obj1.toString;
        }, null);

        trace("obj2 as num: " + (0 + obj2));
        trace(obj2);
        trace("");

        trace("// obj3: methods through __resolve");
        var obj3 = {
            __proto__: null,
            __resolve: function(name) {
                trace("__resolve(" + name + ") called!");
                return name == "valueOf" ? obj1.valueOf
                    : name == "toString" ? obj1.toString
                    : undefined;
            }
        };

        trace("obj3 as num: " + (0 + obj3));
        trace(obj3);

        fscommand("quit");
    }

    static function traceArgs() {
        trace("called with " + arguments.length + " arguments:");
        for (var k in arguments) {
            trace("  " + k + ": " + arguments[k]);
        }
        trace("");
    }
}
