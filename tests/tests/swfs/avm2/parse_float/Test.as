// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {

        }
    }
}

trace("// parseFloat(\"12345\")");
trace(parseFloat("12345"));
trace("// parseFloat(\"012345.67890\")");
trace(parseFloat("012345.67890"));
trace("// parseFloat(\"    99999.99999          \")");
trace(parseFloat(" \t\r\n99999.99999\t\r\n      "));
trace("// parseFloat(\"-22222222222222222\")");
trace(parseFloat("-22222222222222222"));
trace("// parseFloat(\".0000000000000000000000005\")");
trace(parseFloat(".0000000000000000000000005"));
trace("// parseFloat(\"0000.12345GIBBERISH\")");
trace(parseFloat("0000.12345GIBBERISH"));
trace("// parseFloat(\"9e99999\")");
trace(parseFloat("9e99999"));
trace("// parseFloat(\"+100e-100\")");
trace(parseFloat("+100e-100"));
trace("// parseFloat(\"-123.234E+66\")");
trace(parseFloat("-123.234E+66"));
trace("// parseFloat(\".2E20E1\")");
trace(parseFloat(".2E20E1"));
trace("// parseFloat(\"1.2345.678\")");
trace(parseFloat("1.2345.678"));
trace("// parseFloat(\"1.2345.6e50\")");
trace(parseFloat("1.2345.6e50"));
trace("// parseFloat(\"-034.1+e20\")");
trace(parseFloat("-034.1+e20"));
trace("// parseFloat(\"e10\")");
trace(parseFloat("e10"));
trace("// parseFloat(\"BADBAD\")");
trace(parseFloat("BADBAD"));
trace("// parseFloat(\"-\")");
trace(parseFloat("-"));
trace("// parseFloat(\"0xff\")");
trace(parseFloat("0xff"));
trace("// parseFloat(\"Infinity\")");
trace(parseFloat("Infinity"));
trace("// parseFloat(true)");
var b = true;
trace(parseFloat(b));
trace("// parseFloat(1.2)");
var f = 1.2;
trace(parseFloat(f));
trace("// parseFloat(Infinity)");
f = Infinity
trace(parseFloat(f));
trace("// parseFloat({toString})");
var o = {toString:function()
{
   return "5";
}};
trace(parseFloat(o));
trace("// parseFloat(new ClassWithToString())");
class C {
    public function toString(): String { return "6"; }
}
var c = new C();
trace(parseFloat(c));
