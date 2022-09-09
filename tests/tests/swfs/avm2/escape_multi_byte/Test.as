package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {

        }
    }
}

import flash.utils.escapeMultiByte;
import flash.utils.unescapeMultiByte;
import flash.utils.ByteArray;

// For some reason we cannot put the raw alien char in the trace - the output.txt
// which contains 👾, breaks the test framework.
trace("// escapeMultiByte")
trace(escapeMultiByte("👾"));

trace("// escapeMultiByte - stop at 0x00");
var ba = new ByteArray();
ba.writeByte(0x50);
ba.writeByte(0x00);
ba.writeByte(0x50);
trace(escapeMultiByte(ba));

trace("// escapeMultiByte - unpaired surrogate");
ba.clear()
// This is 0xDC00 utf-encoded:
ba.writeByte(0xed);
ba.writeByte(0xb0);
ba.writeByte(0x80);
trace(escapeMultiByte(ba));

trace("// chars 0x01 - 0x7f")
var i:int;
ba.clear();
for(i=1; i < 0x80; i++) {
  ba.writeByte(i);
}
trace(escapeMultiByte(ba));


trace("// unescapeMultiByte - invalid percent sequences");
for each (var s:String in ["", "%", "%A", "%AG", "%GA", "%%", "%A%", "%G%"] ) {
  trace("// " + s);
  trace(unescapeMultiByte(s));
}

// This is 0xDC00 utf-encoded:
ba.clear();
ba.writeByte(0xed);
ba.writeByte(0xb0);
ba.writeByte(0x80);
trace("// unescapeMultiByte, unpaired surrogate")
trace(escape(unescapeMultiByte(ba)));


// Zero char
ba.clear();
ba.writeByte(0x50);
ba.writeByte(0x00);
ba.writeByte(0x50);
trace("// unescapeMultiByte, stops on zero char");
trace(escape(unescapeMultiByte(ba)));

// Zero char after percent
ba.clear();
ba.writeByte(0x25);  // %
ba.writeByte(0x00);
ba.writeByte(0x48);
trace("// unescapeMultiByte, does not stop on zero char after %");
trace(escape(unescapeMultiByte(ba)));

trace("// unescapeMultiByte - P");
trace(unescapeMultiByte("%50"));

trace("// unescapeMultiByte handles lowercase");
trace(unescapeMultiByte("%4f"));


trace("// invader char");
trace(escape(unescapeMultiByte("%F0%9F%91%BE")));

trace("// percent-escaped unpaired surrogate");
trace(escape(unescapeMultiByte("%ED%B0%80")));

trace("// percent-escaped invalid utf-8");
trace(escape(unescapeMultiByte("%ED%B0")));

trace("// percent-escaped invalid utf-8");
trace(escape(unescapeMultiByte("%F0%9F%91")));

trace("// percent-escaped invalid utf-8 but lowercase");
trace(escape(unescapeMultiByte("%f0%9f%91")));

