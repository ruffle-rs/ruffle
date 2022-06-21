// compiled with mxmlc

package {
    public class Test {}
}

var undefined_;

trace("// trace(parseInt());")
trace(parseInt());

trace("// trace(parseInt(undefined_));")
trace(parseInt(undefined_));

trace("// trace(parseInt(undefined_, 32));")
trace(parseInt(undefined_, 32));

trace("// trace(parseInt(\"undefined\", 32));")
trace(parseInt("undefined", 32));

trace("// trace(parseInt(\"\"));")
trace(parseInt(""));

trace("// trace(parseInt(\"123\"));")
trace(parseInt("123"));

trace("// trace(parseInt(\"100\", 10));")
trace(parseInt("100", 10));

trace("// trace(parseInt(\"100\", 0));")
trace(parseInt("100", 0));

trace("// trace(parseInt(\"100\", 1));")
trace(parseInt("100", 1));

trace("// trace(parseInt(\"100\", 2));")
trace(parseInt("100", 2));

trace("// trace(parseInt(\"100\", 36));")
trace(parseInt("100", 36));

trace("// trace(parseInt(\"100\", 37));")
trace(parseInt("100", 37));

trace("// trace(parseInt(\"100\", -1));")
trace(parseInt("100", -1));

trace("// trace(parseInt(\"100\", {}));")
var radix = {};
trace(parseInt("100", radix));

trace("// trace(parseInt(\"100\", true));")
radix = true;
trace(parseInt("100", radix));

trace("// trace(parseInt(\"100\", false));")
radix = false;
trace(parseInt("100", radix));

trace("// trace(parseInt(\"100\", NaN));")
trace(parseInt("100", NaN));

trace("// trace(parseInt(\"100\", undefined_));")
trace(parseInt("100", undefined_));

trace("// trace(parseInt(\"0x123\"));")
trace(parseInt("0x123"));

trace("// trace(parseInt(\"0xabc\"));")
trace(parseInt("0xabc"));

trace("// trace(parseInt(\"010\", 2));")
trace(parseInt("010", 2));

trace("// trace(parseInt(\"-0100\"));")
trace(parseInt("-0100"));

trace("// trace(parseInt(\"-0100z\"));")
trace(parseInt("-0100z"));

trace("// trace(parseInt(\"0x+0X100\"));")
trace(parseInt("0x+0X100"));

trace("// trace(parseInt(123));")
var n = 123;
trace(parseInt(n));

trace("// trace(parseInt(123, 32));")
trace(parseInt(n, 32));

trace("// trace(parseInt(\"++1\"));")
trace(parseInt("++1"));

trace("// trace(parseInt(\"0x100\", 36));")
trace(parseInt("0x100", 36));

trace("// trace(parseInt(\" 0x100\", 36));")
trace(parseInt(" 0x100", 36));

trace("// trace(parseInt(\"0y100\", 36));")
trace(parseInt("0y100", 36));

trace("// trace(parseInt(\" 0y100\", 36));")
trace(parseInt(" 0y100", 36));

trace("// trace(parseInt(\"-0x100\", 36));")
trace(parseInt("-0x100", 36));

trace("// trace(parseInt(\" -0x100\", 36));")
trace(parseInt(" -0x100", 36));

trace("// trace(parseInt(\"-0y100\", 36));")
trace(parseInt("-0y100", 36));

trace("// trace(parseInt(\" -0y100\", 36));")
trace(parseInt(" -0y100", 36));

trace("// trace(parseInt(\"-0x100\"));")
trace(parseInt("-0x100"));

trace("// trace(parseInt(\"0x-100\"));")
trace(parseInt("0x-100"));

trace("// trace(parseInt(\" 0x-100\"));")
trace(parseInt(" 0x-100"));

trace("// trace(parseInt(\"0x -100\"));")
trace(parseInt("0x -100"));

trace("// trace(parseInt(\"-0100\"));")
trace(parseInt("-0100"));

trace("// trace(parseInt(\"0-100\"));")
trace(parseInt("0-100"));

trace("// trace(parseInt(\"+0x123\", 33));")
trace(parseInt("+0x123", 33));

trace("// trace(parseInt(\"+0x123\", 34));")
trace(parseInt("+0x123", 34));

trace("// trace(parseInt(\"0\"));")
trace(parseInt("0"));

trace("// trace(parseInt(\" 0\"));")
trace(parseInt(" 0"));

trace("// trace(parseInt(\" 0 \"));")
trace(parseInt(" 0 "));

trace("// trace(parseInt(\"077\"));")
trace(parseInt("077"));

trace("// trace(parseInt(\"  077\"));")
trace(parseInt("  077"));

trace("// trace(parseInt(\"  077   \"));")
trace(parseInt("  077   "));

trace("// trace(parseInt(\"  -077\"));")
trace(parseInt("  -077"));

trace("// trace(parseInt(\"077 \"));")
trace(parseInt("077 "));

trace("// trace(parseInt(\"11\", 2));")
trace(parseInt("11", 2));

trace("// trace(parseInt(\"11\", 3));")
trace(parseInt("11", 3));

trace("// trace(parseInt(\"11\", 3.8));")
trace(parseInt("11", 3.8));

trace("// trace(parseInt(\"0x12\"));")
trace(parseInt("0x12"));

trace("// trace(parseInt(\"0x12\", 16));")
trace(parseInt("0x12", 16));

trace("// trace(parseInt(\"0x12\", 16.1));")
trace(parseInt("0x12", 16.1));

trace("// trace(parseInt(\"0x12\", NaN));")
trace(parseInt("0x12", NaN));

trace("// trace(parseInt(\"0x  \"));")
trace(parseInt("0x  "));

trace("// trace(parseInt(\"0x\"));")
trace(parseInt("0x"));

trace("// trace(parseInt(\"0x  \", 16));")
trace(parseInt("0x  ", 16));

trace("// trace(parseInt(\"0x\", 16));")
trace(parseInt("0x", 16));

trace("// trace(parseInt(\"12aaa\"));")
trace(parseInt("12aaa"));

trace("// trace(parseInt(\"100000000000000000000000000000000000000000000000000000000000\" + \"00000000000000000000000000000000000000000000000000000000000000000000\" + \"00000000000000000000000000000000000000000000000000000000000000000000\" + \"00000000000000000000000000000000000000000000000000000000000000000000\" + \"00000000000000000000000000000000000000000000000000000000000000000000\" + \"000000000000000\"));")
trace(parseInt("100000000000000000000000000000000000000000000000000000000000" + "00000000000000000000000000000000000000000000000000000000000000000000" + "00000000000000000000000000000000000000000000000000000000000000000000" + "00000000000000000000000000000000000000000000000000000000000000000000" + "00000000000000000000000000000000000000000000000000000000000000000000" + "000000000000000"));

trace("// trace(parseInt(\"0x1000000000000000000000000000000000000000000000000000000000\" + \"00000000000000000000000000000000000000000000000000000000000000000000\" + \"00000000000000000000000000000000000000000000000000000000000000000000\" + \"00000000000000000000000000000000000000000000000000000000000000000000\" + \"00000000000000000000000000000000000000000000000000000000000000000000\" + \"000000000000000\"));")
trace(parseInt("0x1000000000000000000000000000000000000000000000000000000000" + "00000000000000000000000000000000000000000000000000000000000000000000" + "00000000000000000000000000000000000000000000000000000000000000000000" + "00000000000000000000000000000000000000000000000000000000000000000000" + "00000000000000000000000000000000000000000000000000000000000000000000" + "000000000000000"));

trace("// trace(parseInt(String.fromCharCode(0x2000) + \"123\"));")
trace(parseInt(String.fromCharCode(0x2000) + "123"));
