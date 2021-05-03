// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf 
class Test {
    static function main(current) {
        // Variables to bypass MTASC's type checker.
        var number = 12;
        var object = {};
        var string = "bar";

        trace("// not enough args");
        trace(Object.registerClass());

        trace("// too many args");
        trace(Object.registerClass("foo", null, "bar"));

        trace("// register a constructor");
        trace(Object.registerClass("foo", function() {}));

        trace("// unregister a constructor");
        trace(Object.registerClass("foo", null));

        trace("// weird symbols names");
        trace(Object.registerClass(null, null));
        trace(Object.registerClass(number, null));
        trace(Object.registerClass(object, null));

        trace("// wrong types");
        trace(Object.registerClass("foo", number));
        trace(Object.registerClass("foo", string));
        trace(Object.registerClass("foo", object));
    }
}