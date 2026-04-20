// Compile with:
//  mtasc -main -header 200:200:30 -version 8 Test.as -swf test.swf
class Test {
    function Test(mode, ASnew) {
        trace("--- begin " + mode);

        trace("// ASnew()")
        trace(ASnew());

        trace("// ASnew.apply(null, [])");
        trace(ASnew.apply(null, []));

        trace("// ASnew.call(null)");
        trace(ASnew.call(null));

        trace("// (() => ASnew())()");
        var wrapper = function() {
            return ASnew();
        };
        trace(wrapper());

        var obj = {};
        trace("// obj.ASnew (property)");
        obj.addProperty("ASnew", ASnew, null);
        trace(obj.ASnew);

        trace("// obj.__resolve = ASnew; obj.foo");
        obj.__resolve = ASnew;
        trace(obj.foo);

        trace("// (new Date({ valueOf: ASnew })).getTime()");
        obj = { valueOf: ASnew };
        trace((new Date(obj)).getTime());
        
        trace("--- end " + mode);
    }

    static function main(current) {
        trace("// ASnew = ASnative(2, 0)");
        var ASnew = _global.ASnative(2, 0);
        trace(ASnew);

        var Test = Test; // silences MTASC's type checker
        Test("function", ASnew);
        var discard = new Test("constructor", ASnew);

        fscommand("quit");
    }
}
