// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {
        }
    }
}

import flash.utils.ByteArray;

trace("//Vector.<int>([1, 2, 3])")
var v = Vector.<int>([1, 2, 3]);
trace(v);
trace("//Vector.<int>(vec)")
v = Vector.<int>(v);
trace(v);
trace("//Vector.<int>(bytearray)")
var bytes = new ByteArray();
bytes.writeFloat(123.456);
v = Vector.<int>(bytes);
trace(v);

trace()
trace("//Object()")
var cls = Object;
var o = cls();
trace(o);

trace()
trace("//Object(\"asdf\")")
o = cls("asdf");
trace(o);

trace()
trace("//String(123)")
var s = String(123);
trace(s);
trace(typeof s);

trace()
trace("//RegExp(\"asdf\")")
cls = RegExp;
var pat = cls("asdf");
trace(pat);
trace("//RegExp(\"asdf\", \"gi\")")
var pat = cls("asdf", "gi");
trace(pat);
trace("//RegExp(regexp)")
pat = cls(pat);
trace(pat);


