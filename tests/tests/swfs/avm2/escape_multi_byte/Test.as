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