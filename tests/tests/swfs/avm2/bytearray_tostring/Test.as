package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.utils.ByteArray;

var ba = new ByteArray();
ba.writeUTFBytes("\uFEFFabc");

trace("ba.toString():", ba.toString());
trace("ba.toString().length:", ba.toString().length);
trace("ba.position:", ba.position);

ba.position = 0;
trace("ba.position:", ba.position);
trace("ba.toString():", ba.toString());
trace("ba.toString().length:", ba.toString().length);
trace("ba.position:", ba.position);

// Verify BOM was written.
trace("ba.readUnsignedByte().toString(16):", ba.readUnsignedByte().toString(16));
trace("ba.readUnsignedByte().toString(16):", ba.readUnsignedByte().toString(16));
trace("ba.readUnsignedByte().toString(16):", ba.readUnsignedByte().toString(16));

var ba2 = new ByteArray();
ba2.writeUTFBytes("hello");
ba2.writeByte(0x00);
ba2.writeUTFBytes("world");

// Flash's trace seems to strip \u0000, but the length is correct.
// trace("ba2.toString():", ba2.toString());
trace("ba2.toString().length:", ba2.toString().length);
trace("ba2.position:", ba2.position);

ba2.position = 0;
trace("ba2.position:", ba2.position);
// trace("ba2.toString():", ba2.toString());
trace("ba2.toString().length:", ba2.toString().length);
trace("ba2.position:", ba2.position);
