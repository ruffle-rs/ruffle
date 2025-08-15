// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf 
class Test {
    static function main(current) {
        trace("// Number(10)");
        show(Number(10));
        trace("// new Number(42)");
        var num = show(new Number(42));
        showKeys(num);
        trace("");

        trace("// Boolean(false)");
        show(Boolean(false));
        trace("// new Boolean(true)");
        var bool = show(new Boolean(true));
        showKeys(bool);
        trace("");

        trace("// String('bluh')");
        show(String('bluh'));
        trace("// new String('blah')");
        var str = show(new String('blah'));
        showKeys(str);
        trace("length = " + str.length);
        trace("hasOwnProperty = " + str.hasOwnProperty("length"));

        trace("// str.length = 'hmm'");
        str.length = 'hmm';
        trace("length = " + str.length);
        showKeys(str);
        trace("// delete str.length");
        delete str.length;
        trace("length = " + str.length);
    }

    static function show(val) {
        trace("typeof = " + (typeof val) + ", value = " + val);
        return val;
    }

    static function showKeys(obj) {
        var keys = []
        for (var key in obj) {
            keys.push(key);
        }
        trace("keys = [" + keys + "]");
        return obj;
    }
}